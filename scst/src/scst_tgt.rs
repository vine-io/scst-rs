use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

use crate::handler::Handler;
use crate::target::Driver;
use crate::{Layer, ScstError, read_dir, read_fl};

static SCST_ROOT_OLD: &str = "/sys/kernel/scst_tgt";
static SCST_ROOT_NEW: &str = "/sys/devices/scst";
static SCST_HANDLER: &str = "handlers";
static SCST_DRIVER: &str = "targets";

#[derive(Debug)]
pub struct Scst {
    root: String,
    version: String,

    handlers: HashMap<String, Handler>,
    iscsi_driver: Driver,
}

impl Scst {
    pub fn init() -> Result<Self> {
        let mut scst_root = Path::new(SCST_ROOT_NEW);
        if !scst_root.exists() {
            scst_root = Path::new(SCST_ROOT_OLD);
            if !scst_root.exists() {
                anyhow::bail!(ScstError::NoModule);
            }
        }

        let handlers = HashMap::new();
        let mut scst = Scst {
            root: scst_root.to_string_lossy().to_string(),
            version: "".to_string(),
            handlers,
            iscsi_driver: Driver::default(),
        };
        scst.load(scst_root)?;

        Ok(scst)
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn handlers(&self) -> &HashMap<String, Handler> {
        &self.handlers
    }

    pub fn get_handler<S: AsRef<str>>(&self, name: S) -> Result<&Handler> {
        self.handlers
            .get(name.as_ref())
            .context(ScstError::NoHandler(name.as_ref().to_string()))
    }

    pub fn get_handler_mut<S: AsRef<str>>(&mut self, name: S) -> Result<&mut Handler> {
        self.handlers
            .get_mut(name.as_ref())
            .context(ScstError::NoHandler(name.as_ref().to_string()))
    }

    pub fn iscsi(&self) -> &Driver {
        &self.iscsi_driver
    }

    pub fn iscsi_mut(&mut self) -> &mut Driver {
        &mut self.iscsi_driver
    }
}

impl Layer for Scst {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.version = read_fl(root_ref.join("version"))?;

        // traverse handler directory
        self.handlers = read_dir(root_ref.join(SCST_HANDLER))?
            .filter_map(|res| res.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let mut handler = Handler::default();
                handler.load(entry.path()).ok();
                Some((handler.name().to_string(), handler))
            })
            .collect();

        // traverse driver directory
        let mut iscsi_driver = Driver::default();
        iscsi_driver
            .load(root_ref.join(SCST_DRIVER).join("iscsi"))
            .map_err(|e| ScstError::Unknown(e))?;
        self.iscsi_driver = iscsi_driver;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use regex::Regex;

    use super::Result;

    #[test]
    fn it_works() -> Result<()> {
        let re = Regex::new(r"(\d+):(\d+):(\d+):(\d+)")?;
        assert!(re.is_match("0:0:0:0"));
        assert!(re.is_match("0:11:3:4"));
        assert!(re.is_match("023:11:3:4"));
        Ok(())
    }
}
