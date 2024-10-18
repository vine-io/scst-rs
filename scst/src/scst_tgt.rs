use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::handler::Handler;
use crate::target::Driver;
use crate::{Config, CopyManager, Layer, Options, ScstError, read_dir, read_fl};

static SCST_ROOT_OLD: &str = "/sys/kernel/scst_tgt";
static SCST_ROOT_NEW: &str = "/sys/devices/scst";
static SCST_HANDLER: &str = "handlers";
static SCST_DRIVER: &str = "targets";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Scst {
    root: String,
    version: String,

    handlers: BTreeMap<String, Handler>,
    iscsi_driver: Driver,
    copy_driver: CopyManager,
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
            copy_driver: CopyManager::default(),
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

    /// add a device for handler.
    ///
    /// ```no_run
    /// use scst::{Scst, Options};
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let mut options = Options::new();
    /// options.insert("read_only", "1");
    ///
    /// scst.add_device("vdisk_blockio", "disk1", "/dev/sdb", &options)?;
    /// ```
    pub fn add_device<S: AsRef<str>>(
        &mut self,
        handler: S,
        name: S,
        filename: S,
        options: &Options,
    ) -> Result<()> {
        let handler_ref = handler.as_ref();
        let name_ref = name.as_ref();

        let handler = self.get_handler_mut(handler_ref)?;
        handler.add_device(name_ref, filename.as_ref(), options)?;

        self.copy_driver
            .load(self.copy_driver.root().to_path_buf())?;

        Ok(())
    }

    /// delete device for handler
    ///
    /// ```no_run
    /// use scst::{Scst, Options};
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// scst.del_device("vdisk_blockio", "disk1")?;
    /// ```
    pub fn del_device<S: AsRef<str>>(&mut self, handler: S, name: S) -> Result<()> {
        let handler_ref = handler.as_ref();
        let handler = self.get_handler_mut(handler_ref)?;

        handler.del_device(name.as_ref())?;

        self.copy_driver
            .load(self.copy_driver.root().to_path_buf())?;

        Ok(())
    }
}

impl Scst {
    /// loads scst configuration scst from `Config`
    /// ```no_run
    /// use anyhow::Result;
    /// use scst::Scst;
    ///
    /// fn main() -> Result<()> {
    ///     let mut scst = Scst::init()?;
    ///
    ///     let cfg = Config::read("/tmp/scst.yml")?;
    ///     scst1.from_cfg(&cfg)?;
    ///
    ///     Ok(())
    /// }
    /// ```
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
            if dc.enabled() == 1 {
                driver.enable()?;
            }

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

                if tc.enabled() == 1 {
                    target.enable()?
                }
            }

            self.copy_driver
                .load(self.copy_driver.root().to_path_buf())?;
        }

        Ok(())
    }

    /// converts scst information to `Config`
    /// ```no_run
    /// use anyhow::Result;
    /// use scst::Scst;
    ///
    /// fn main() -> Result<()> {
    ///     let mut scst = Scst::init()?;
    ///
    ///     let cfg = scst.to_cfg();
    ///     cfg.write_to("/tmp/scst.yml")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn to_cfg(&self) -> Config {
        Config::new(
            &self.handlers(),
            &[self.iscsi()],
            &self.copy_driver,
            self.version(),
        )
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

        let mut copy_driver = CopyManager::default();
        copy_driver
            .load(root_ref.join(SCST_DRIVER).join("copy_manager"))
            .map_err(|e| ScstError::Unknown(e))?;
        self.copy_driver = copy_driver;

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
