use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;

mod config;
mod device;
mod error;
mod handler;
mod scst_tgt;
mod stat;
mod target;

pub use config::*;
pub use device::*;
pub use error::*;
pub use handler::*;
pub use scst_tgt::*;
pub use stat::*;
pub use target::*;

pub(crate) trait Layer {
    fn root(&self) -> &Path;

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()>;

    fn mgmt<S: AsRef<OsStr>>(&mut self, root: S, cmd: S) -> Result<()> {
        let mgmt = Path::new(root.as_ref()).join("mgmt");
        // println!(
        //     "echo \"{}\" > {}",
        //     cmd.as_ref().to_string_lossy(),
        //     mgmt.to_string_lossy()
        // );
        echo(mgmt.as_ref(), cmd.as_ref())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Options {
    inner: HashMap<String, String>,
}

impl Options {
    pub fn new() -> Self {
        Options {
            inner: HashMap::new(),
        }
    }

    pub fn insert<S: AsRef<str>>(&mut self, k: S, v: S) -> &Self {
        self.inner
            .insert(k.as_ref().to_string(), v.as_ref().to_string());
        self
    }

    pub fn contains_keys<'a>(&self, keys: &'a [String]) -> Vec<&'a str> {
        keys.iter()
            .filter(|key| self.inner.contains_key(*key))
            .filter_map(|key| Some(key.as_str()))
            .collect::<Vec<&str>>()
    }

    pub fn different_set(&self, keys: &[String]) -> Vec<&str> {
        self.inner
            .keys()
            .filter_map(|key| {
                if !keys.contains(key) {
                    Some(key.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>()
    }

    /// packs Options, converts Options to String. return None if field 'inner' is empty.
    ///
    /// ```no_run
    /// use scst::Options;
    ///
    /// let mut opt = Options::new();
    /// let s = opt.pack();
    /// assert_eq!(s, None);
    ///
    /// opt.insert("a", "b");
    /// assert_eq!(opt.pack(), Some("a=b".to_string()));
    /// ```
    pub fn pack(&self) -> Option<String> {
        let slice = self
            .inner
            .iter()
            .filter_map(|(key, value)| Some(key.to_owned() + "=" + value))
            .collect::<Vec<String>>();

        if slice.len() > 0 {
            return Some(slice.join(";"));
        }

        None
    }

    /// like `pack()`, but checks input firstly.
    ///
    /// ```no_run
    /// use scst::Options;
    ///
    /// let mut opt = Options::new();
    /// let s = opt.check_pack(&Vec::new());
    /// assert_eq!(s.unwrap(), None);
    ///
    /// opt.insert("a", "b");
    /// assert!(opt.check_pack(&["c".to_string()]).is_err());
    /// ```
    pub fn check_pack(&self, keys: &[String]) -> Result<Option<String>> {
        let sets = self
            .inner
            .keys()
            .filter_map(|key| {
                if !keys.contains(key) {
                    Some(key.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>();
        if sets.len() > 0 {
            return Err(anyhow::anyhow!("invalid paramsters [{}]", sets.join(",")));
        }

        Ok(self.pack())
    }
}

pub(crate) fn read_fl<P: AsRef<Path>>(path: P) -> Result<String> {
    let text = fs::read_to_string(path)?
        .split('\n')
        .collect::<Vec<&str>>()
        .get(0)
        .map(|s| s.to_string())
        .unwrap_or("".to_string());

    Ok(text)
}

pub(crate) fn read_dir<P: AsRef<Path>>(path: P) -> Result<fs::ReadDir> {
    let read_dir = fs::read_dir(path).map_err(|e| ScstError::Io(e))?;
    Ok(read_dir)
}

pub(crate) fn read_link<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let buf = fs::read_link(path).map_err(|e| ScstError::Io(e))?;
    Ok(buf)
}

pub(crate) fn echo<S: AsRef<OsStr>>(root: S, cmd: S) -> Result<()> {
    let cmd_str = cmd.as_ref().to_string_lossy();
    let mut fd = fs::File::create(Path::new(root.as_ref()))?;
    fd.write(cmd_str.as_bytes()).map_err(|e| ScstError::Io(e))?;

    Ok(())
}

pub(crate) fn cmd_with_options(
    cmd: &str,
    params: &Vec<String>,
    options: &Options,
) -> Result<String> {
    let cmd_string = cmd.to_string();
    let out = options
        .check_pack(&params)?
        .and_then(|s| {
            let mut c = cmd_string.clone();
            c.push_str(" ");
            c.push_str(&s);
            Some(c)
        })
        .or(Some(cmd_string))
        .unwrap();

    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn foo_file(text: &str) -> Result<String> {
        let path = std::env::temp_dir().join("foo.txt");
        let mut file = fs::File::create(path.clone())?;
        file.write_all(text.as_bytes())?;
        Ok(path.to_string_lossy().to_string())
    }

    #[test]
    fn test_read_fl() -> Result<()> {
        let fl1 = foo_file("1")?;
        assert_eq!(read_fl(fl1)?, "1");

        let fl2 = foo_file("3.1\nDEBUG")?;
        assert_eq!(read_fl(fl2)?, "3.1");

        let fl3 = foo_file("open\n[key]")?;
        assert_eq!(read_fl(fl3)?, "open");

        Ok(())
    }

    #[test]
    pub fn test_options() -> Result<()> {
        let mut opt = Options::new();
        let s = opt.pack();
        assert_eq!(s, None);

        let s = opt.check_pack(&Vec::new());
        assert_eq!(s.unwrap(), None);

        opt.insert("a", "b");
        assert_eq!(opt.pack(), Some("a=b".to_string()));

        opt.insert("a", "b");
        assert!(opt.check_pack(&["c".to_string()]).is_err());

        Ok(())
    }
}
