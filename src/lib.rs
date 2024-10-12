use anyhow::{Ok, Result};
use std::{fs, path::Path};

pub mod scst_tgt;

trait Layer {
    fn root(&self) -> &Path;

    fn open<P>(&mut self, root: P) -> Result<()>
    where
        P: AsRef<Path>;
}

pub(crate) fn read_fl<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let text = fs::read_to_string(path)?
        .split('\n')
        .collect::<Vec<&str>>()
        .get(0)
        .map(|s| s.to_string())
        .unwrap_or("".to_string());

    Ok(text)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_read_fl() -> Result<()> {
        let fl1 = rand_touch_file("1")?;
        assert_eq!(read_fl(fl1)?, "1");

        let fl2 = rand_touch_file("3.1\nDEBUG")?;
        assert_eq!(read_fl(fl2)?, "3.1");

        let fl3 = rand_touch_file("open\n[key]")?;
        assert_eq!(read_fl(fl3)?, "open");

        Ok(())
    }

    fn rand_touch_file(text: &str) -> Result<String> {
        let path = std::env::temp_dir().join("foo.txt");
        let mut file = fs::File::create(path.clone())?;
        file.write_all(text.as_bytes())?;
        Ok(path.to_string_lossy().to_string())
    }
}
