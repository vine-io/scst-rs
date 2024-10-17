use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Ok, Result};

use crate::handler::Handler;
use crate::target::Driver;
use crate::{Config, Layer, Options, ScstError, read_dir, read_fl};

static SCST_ROOT_OLD: &str = "/sys/kernel/scst_tgt";
static SCST_ROOT_NEW: &str = "/sys/devices/scst";
static SCST_HANDLER: &str = "handlers";
static SCST_DRIVER: &str = "targets";

#[derive(Debug)]
pub struct Scst {
    root: String,
    version: String,

    handlers: BTreeMap<String, Handler>,
    iscsi_driver: Driver,
}

impl Scst {
    /// initizatation scst
    /// ```no_run
    /// use scst::Scst;
    ///
    /// let scst = Scst::init()?:
    /// ```
    pub fn init() -> Result<Self> {
        let mut scst_root = Path::new(SCST_ROOT_OLD);
        if !scst_root.exists() {
            scst_root = Path::new(SCST_ROOT_NEW);
            if !scst_root.exists() {
                anyhow::bail!(ScstError::NoModule);
            }
        }

        let mut scst = Scst {
            root: scst_root.to_string_lossy().to_string(),
            version: "".to_string(),
            handlers: BTreeMap::new(),
            iscsi_driver: Driver::default(),
        };
        scst.load(scst_root)?;

        Ok(scst)
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn handlers(&self) -> Vec<&Handler> {
        self.handlers.values().collect()
    }

    /// get scst handler
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

    /// get iscsi driver
    pub fn iscsi(&self) -> &Driver {
        &self.iscsi_driver
    }

    pub fn iscsi_mut(&mut self) -> &mut Driver {
        &mut self.iscsi_driver
    }
}

impl Scst {
    pub fn from_cfg(&mut self, cfg: &Config) -> Result<()> {
        for hc in cfg.handlers() {
            let handler = self.get_handler_mut(hc.name())?;
            for dev in hc.devices() {
                if handler.get_device(dev.name()).is_err() {
                    let opts = Options::new();
                    handler.add_device(dev.name(), dev.filename(), &opts)?
                }
            }
        }

        for dc in cfg.drivers() {
            let driver = { self.iscsi_mut() };
            for tc in dc.targets() {
                let target = {
                    let mut res = driver.get_target_mut(tc.name());
                    if res.is_err() {
                        let opts = Options::new();
                        res = driver.add_target(tc.name(), &opts);
                    }
                    res?
                };

                for lc in tc.luns() {
                    let name = format!("lun {}", lc.id());
                    if target.get_lun(&name).is_err() {
                        let opts = Options::new();
                        target.add_lun(lc.device(), lc.id(), &opts)?;
                    }
                }

                for gc in tc.groups() {
                    let group = {
                        let mut res = target.get_ini_group_mut(gc.name());
                        if res.is_err() {
                            res = target.create_ini_group(gc.name());
                        }
                        res?
                    };

                    for lc in gc.luns() {
                        let name = format!("lun {}", lc.id());
                        if group.get_lun(&name).is_err() {
                            let opts = Options::new();
                            group.add_lun(lc.device(), lc.id(), &opts)?;
                        }
                    }

                    for ini in gc.initiators() {
                        if !group.initiators().contains(&ini.to_string()) {
                            group.add_initiator(ini.to_string())?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn to_cfg(&self) -> Config {
        Config::new(&self.handlers(), &[self.iscsi()], self.version())
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
