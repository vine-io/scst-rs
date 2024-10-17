use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::device::Device;
use crate::{Layer, Options, ScstError, cmd_with_options, read_dir, read_fl};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Handler {
    #[serde(skip)]
    root: String,
    name: String,
    r#type: String,

    devices: BTreeMap<String, Device>,
}

impl Handler {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_type(&self) -> &str {
        &self.r#type
    }

    pub fn devices(&self) -> Vec<&Device> {
        self.devices.values().collect()
    }

    pub fn get_device<S: AsRef<str>>(&self, name: S) -> Result<&Device> {
        self.devices
            .get(name.as_ref())
            .context(ScstError::NoDevice(name.as_ref().to_string()))
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
    /// scst.get_handler_mut("vdisk_blockio")?
    ///   .add_device("disk1", "/dev/sdb", &options)?;
    /// ```
    pub fn add_device<S: AsRef<str>>(
        &mut self,
        name: S,
        filename: S,
        options: &Options,
    ) -> Result<()> {
        let name_ref = name.as_ref();
        if self.devices.contains_key(name_ref) {
            anyhow::bail!(ScstError::DeviceExists(name_ref.to_string()))
        }

        let root = self.root().to_path_buf();
        let mut cmd = format!("add_device {} filename={}", name_ref, filename.as_ref());
        let params = vec![
            "active".to_string(),
            "bind_alua_state".to_string(),
            "blocksize".to_string(),
            "cluster_mode".to_string(),
            "dif_filename".to_string(),
            "dif_mode".to_string(),
            "dif_static_app_tag".to_string(),
            "dif_type".to_string(),
            "filename".to_string(),
            "numa_node_id".to_string(),
            "nv_cache".to_string(),
            "read_only".to_string(),
            "removable".to_string(),
            "rotational".to_string(),
            "thin_provisioned".to_string(),
            "tst".to_string(),
            "write_through".to_string(),
        ];

        cmd = cmd_with_options(&cmd, &params, &options)?;

        self.mgmt(root, cmd.into())
            .map_err(|e| ScstError::DeviceAddFail {
                name: name_ref.to_string(),
                e,
            })?;

        let mut device = Device::default();
        device.load(self.root().join(name_ref))?;
        self.devices.insert(device.name().to_string(), device);

        Ok(())
    }

    /// delete device for handler
    ///
    /// ```no_run
    /// use scst::{Scst, Options};
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// scst.get_handler_mut("vdisk_blockio")?.del_device("disk1")?;
    /// ```
    pub fn del_device<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        let name_ref = name.as_ref();
        if !self.devices.contains_key(name_ref) {
            anyhow::bail!(ScstError::NoDevice(name_ref.to_string()))
        }

        let root = self.root().to_path_buf();
        let cmd = format!("del_device {}", name_ref);
        self.mgmt(root, cmd.into())?;

        self.devices.remove(name_ref);

        Ok(())
    }
}

impl Layer for Handler {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.name = root_ref
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();
        self.root = root_ref.to_string_lossy().to_string();
        self.r#type = read_fl(root_ref.join("type"))?;

        // traverse device directory
        self.devices = read_dir(root_ref)?
            .filter_map(|res| res.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let mut device = Device::default();
                device.load(entry.path()).ok();
                Some((device.name().to_string(), device))
            })
            .collect();

        Ok(())
    }
}
