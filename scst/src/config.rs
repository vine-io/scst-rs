use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Device, Driver, Handler, IniGroup, Lun, Target};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    version: String,
    #[serde(default)]
    handlers: BTreeMap<String, HanderCfg>,
    #[serde(default)]
    drivers: BTreeMap<String, DriverCfg>,
}

impl Config {
    pub(crate) fn new(handlers: &[&Handler], drivers: &[&Driver], version: &str) -> Self {
        let handlers = handlers
            .iter()
            .filter_map(|h| {
                let handler = HanderCfg::from(*h);
                Some((handler.name().to_string(), handler))
            })
            .collect();

        let drivers = drivers
            .iter()
            .filter_map(|h| {
                let handler = DriverCfg::from(*h);
                Some((handler.name().to_string(), handler))
            })
            .collect();

        Config {
            version: version.to_string(),
            handlers,
            drivers,
        }
    }

    /// create `Config` from yaml string
    pub fn from(s: &str) -> Result<Config> {
        let config = serde_yml::from_str::<Config>(s)?;
        Ok(config)
    }

    /// create `Config` from yaml file
    pub fn read<S: AsRef<Path>>(filename: S) -> Result<Config> {
        let s = fs::read_to_string(filename)?;
        Config::from(&s)
    }

    /// encodes `Config` to yaml string
    pub fn to_yml(&self) -> Result<String> {
        let s = serde_yml::to_string(self)?;
        Ok(s)
    }

    /// echo `Config` yaml string to the file
    pub fn write_to<S: AsRef<Path>>(&self, filename: S) -> Result<()> {
        let yml = self.to_yml()?;
        fs::write(filename, yml)?;

        Ok(())
    }

    pub fn handlers(&self) -> Vec<&HanderCfg> {
        self.handlers.values().collect()
    }

    pub fn drivers(&self) -> Vec<&DriverCfg> {
        self.drivers.values().collect()
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HanderCfg {
    #[serde(default)]
    name: String,
    #[serde(default)]
    devices: BTreeMap<String, DeviceCfg>,
}

impl HanderCfg {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn devices(&self) -> Vec<&DeviceCfg> {
        self.devices.values().collect()
    }
}

impl From<&Handler> for HanderCfg {
    fn from(value: &Handler) -> Self {
        let devices = value
            .devices()
            .iter()
            .filter_map(|device| {
                let dc = DeviceCfg::from(*device);
                Some((dc.name.to_string(), dc))
            })
            .collect();

        HanderCfg {
            name: value.name().to_string(),
            devices,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeviceCfg {
    #[serde(default)]
    name: String,
    #[serde(default)]
    filename: String,
    #[serde(default)]
    size: usize,
}

impl DeviceCfg {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl From<&Device> for DeviceCfg {
    fn from(value: &Device) -> Self {
        DeviceCfg {
            name: value.name().to_string(),
            filename: value.filename().to_string_lossy().to_string(),
            size: value.size(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DriverCfg {
    #[serde(default)]
    name: String,
    #[serde(default)]
    enabled: i8,
    #[serde(default)]
    targets: BTreeMap<String, TargetCfg>,
}

impl DriverCfg {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> i8 {
        self.enabled
    }

    pub fn targets(&self) -> Vec<&TargetCfg> {
        self.targets.values().collect()
    }
}

impl From<&Driver> for DriverCfg {
    fn from(value: &Driver) -> Self {
        let targets = value
            .targets()
            .iter()
            .filter_map(|target| {
                let tc = TargetCfg::from(*target);
                Some((tc.name.to_string(), tc))
            })
            .collect();

        DriverCfg {
            name: value.name().to_string(),
            enabled: value.enabled_i8(),
            targets,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TargetCfg {
    #[serde(default)]
    name: String,
    #[serde(default)]
    enabled: i8,
    #[serde(default)]
    rel_tgt_id: u64,

    #[serde(default)]
    luns: Vec<LunCfg>,
    #[serde(default)]
    groups: BTreeMap<String, IniGroupCfg>,
}

impl TargetCfg {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> i8 {
        self.enabled
    }

    pub fn rel_tgt_id(&self) -> u64 {
        self.rel_tgt_id
    }

    pub fn luns(&self) -> Vec<&LunCfg> {
        self.luns.iter().collect()
    }

    pub fn groups(&self) -> Vec<&IniGroupCfg> {
        self.groups.values().collect()
    }
}

impl From<&Target> for TargetCfg {
    fn from(value: &Target) -> Self {
        let luns = value
            .luns()
            .iter()
            .filter_map(|lun| {
                let lc = LunCfg::from(*lun);
                Some(lc)
            })
            .collect();

        let groups = value
            .ini_groups()
            .iter()
            .filter_map(|group| {
                let gc = IniGroupCfg::from(*group);
                Some((gc.name.to_string(), gc))
            })
            .collect();

        TargetCfg {
            name: value.name().to_string(),
            enabled: value.enabled_i8(),
            rel_tgt_id: value.rel_tgt_id(),
            luns,
            groups,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IniGroupCfg {
    #[serde(default)]
    name: String,

    #[serde(default)]
    luns: Vec<LunCfg>,

    #[serde(default)]
    initiators: Vec<String>,
}

impl IniGroupCfg {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn luns(&self) -> Vec<&LunCfg> {
        self.luns.iter().collect()
    }

    pub fn initiators(&self) -> Vec<&str> {
        self.initiators
            .iter()
            .filter_map(|s| Some(s.as_str()))
            .collect()
    }
}

impl From<&IniGroup> for IniGroupCfg {
    fn from(value: &IniGroup) -> Self {
        let luns = value
            .luns()
            .iter()
            .filter_map(|lun| {
                let lc = LunCfg::from(*lun);
                Some(lc)
            })
            .collect();

        let initiators = value
            .initiators()
            .iter()
            .filter_map(|s| Some(s.clone()))
            .collect();

        IniGroupCfg {
            name: value.name().to_string(),
            luns,
            initiators,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LunCfg {
    #[serde(default)]
    id: u64,

    #[serde(default)]
    device: String,
}

impl LunCfg {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn device(&self) -> &str {
        &self.device
    }
}

impl From<&Lun> for LunCfg {
    fn from(value: &Lun) -> Self {
        LunCfg {
            id: value.id(),
            device: value.device().to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::Config;

    #[test]
    fn test_config_from_yaml() -> Result<()> {
        let s = r#"
version: '3.7.0'
handlers:
  dev_cdrom:
    devices: {}
  dev_disk:
    devices: {}
  dev_disk_perf:
    devices: {}
  dev_processor:
    devices: {}
  scst_user:
    devices: {}
  vcdrom:
    devices: {}
  vdisk_blockio:
    devices:
      vol:
        filename: /dev/zvol/tank/vol
        size: 10737418240
  vdisk_fileio:
    devices: {}
  vdisk_nullio:
    devices: {}
drivers:
  iscsi:
    enabled: 1
    targets:
      iqn.2018-11.com.vine:vol:
        enabled: 1
        rel_tgt_id: 0
        luns: []
        groups:
          vol:
            luns:
            - id: 0
              device: vol
            initiators:
            - iqn.1988-12.com.oracle:d4ebaa45254b
"#;

        Config::from(s)?;
        Ok(())
    }
}
