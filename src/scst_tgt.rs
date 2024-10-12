use crate::{read_fl, Layer};

use anyhow::{anyhow, Result};

use std::path::Path;

static SCST_ROOT_OLD: &str = "/sys/kernel/scst_tgt";
static SCST_ROOT_NEW: &str = "/sys/devices/scst";

#[derive(Debug)]
pub struct Scst {
    root: String,
    version: String,
}

impl Scst {
    pub fn init() -> Result<Self> {
        let mut scst_root = Path::new(SCST_ROOT_NEW);
        if !scst_root.exists() {
            scst_root = Path::new(SCST_ROOT_OLD);
            if !scst_root.exists() {
                return Err(anyhow!("iscsi scst not exists"));
            }
        }

        let mut scst = Scst {
            root: scst_root.to_string_lossy().to_string(),
            version: "".to_string(),
        };
        scst.open(scst_root)?;

        Ok(scst)
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Layer for Scst {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn open<P>(&mut self, root: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.version = read_fl(root.as_ref().join("version"))?;

        Ok(())
    }
}
