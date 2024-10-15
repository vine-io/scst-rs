use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use thiserror::Error;

mod device;
mod handler;
mod scst_tgt;
mod target;

pub use device::*;
pub use handler::*;
pub use scst_tgt::*;
pub use target::*;

#[derive(Error, Debug)]
pub enum ScstError {
    #[error("No such SCST module exists")]
    NoModule,
    #[error("A fatal error occured. See \"dmesg\" for more information.")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),

    #[error("Bad attributes given for SCST.")]
    BadAttrs,
    #[error("SCST attribute '{0}' specified is static")]
    AttrStatic(String),
    #[error("Failed to set a SCST attribute '{0}'. See \"demsg\" for more information.")]
    SetattrFail(String),

    #[error("No such handler '{0}' exists.")]
    NoHandler(String),
    #[error("Bad attributes given for handler.")]
    HandlerBadAttr,
    #[error("Handler attribute '{0}' given is static.")]
    HandlerAttrStatic(String),
    #[error("Failed to set handler attribute '{0}'. See \"dmesg\" for more information.")]
    HandlerSetAttrFail(String),

    #[error("No such device '{0}' exists.")]
    NoDevice(String),
    #[error("Device '{0}' already exists.")]
    DeviceExists(String),
    #[error("Failed to add device '{0}'. See \"dmesg\" for more information.")]
    DeviceAddFail(String),
    #[error("Failed to remove device '{0}'. See \"dmesg\" for more information.")]
    DeviceRemFail(String),
    #[error("Bad attributes given for device.")]
    DeviceBadAttr,
    #[error("Device attribute '{0}' specified is static.")]
    DeviceAttrStatic(String),
    #[error("Failed to set device attribute '{0}'. See \"dmesg\" for more information.")]
    DeviceSetAttrFail(String),

    #[error("No such driver '{0}' exists.")]
    NoDriver(String),
    #[error("Driver is incapable of dynamically adding/removing targets or attributes.")]
    DriverNotVirt,
    #[error("Failed to add driver dynamic attribute '{0}'. See \"dmesg\" for more information.")]
    DriverAddAttrFail(String),
    #[error("Failed to remove driver dymanic attribute '{0}'. See \"dmesg\" for more information.")]
    DriverRemAttrFail(String),
    #[error("Bad attributes given for driver.")]
    DriverBadAttrs,
    #[error("Driver attribute '{0}' specified is static.")]
    DriverAttrStatic(String),
    #[error("Failed to set driver attribute '{0}'. See \"dmesg\" for more information.")]
    DriverSetAttrFail(String),

    #[error("No such target '{0}' exists.")]
    NoTarget(String),
    #[error("Target '{0}' already exists.")]
    TargetExists(String),
    #[error("Failed to add target '{0}'. See \"dmesg\" for more information.")]
    TargetAddFail(String),
    #[error("Failed to remove target '{0}'. See \"dmesg\" for more information.")]
    TargetRemFail(String),
    #[error("Failed to set target attribute '{0}'. See \"dmesg\" for more information.")]
    TargetSetAttr(String),
    #[error("Failed to add target dynamic attribute '{0}'. See \"dmesg\" for more information.")]
    TargetAddAttrFail(String),
    #[error("Failed to remove target dynamic attribute. See \"dmesg\" for more information.")]
    TargetRemAttrFail(String),
    #[error("No such LUN '{0}' exists.")]
    TargetNoLun(String),
    #[error("Failed to add LUN '{0}' to target. See \"dmesg\" for more information.")]    
    TargetAddLunFail(String),
    #[error("Failed to remove LUN '{0}' to target. See \"dmesg\" for more information.")]    
    TargetRemLunFail(String),
    #[error("LUN already '{0}' exists.")]
    TargetLunExists(String),
    #[error("Bad attributes given for target.")]
    TargetBadAttrs,
    #[error("Target attribute '{0}' specified is static.")]
    TargetBadAttr(String),
    #[error("Failed to set target attribute '{0}'. See \"dmesg\" for more information.")]
    TargetSetAttrFail(String),
    #[error("Failed to clear LUNs from target. See \"dmesg\" for more information.")]
    TargetClearLunFail,
    #[error(
        "Failed to remove target - target has active sessions. See \"dmesg\" for more information."
    )]
    TargetBusy,

    #[error("No such group '{0}' exists.")]
    NoGroup(String),
    #[error("Group '{0}' already exists.")]
    GroupExists(String),
    #[error("Failed to add group '{0}'. See \"dmesg\" for more information.")]
    GroupAddFail(String),
    #[error("Failed to remove group '{0}'. See \"dmesg\" for more information.")]
    GroupRemFail(String),
    #[error("No such LUN '{0}' exists.")]
    GroupNoLun(String),
    #[error("LUN '{0}' already exists.")]
    GroupLunExists(String),
    #[error("Failed to add LUN '{0}' to group. See \"dmesg\" for more information.")]
    GroupAddLunFail(String),
    #[error("Failed to remove LUN '{0}'. See \"dmesg\" for more information.")]
    GroupRemLunFail(String),
    #[error("Failed to clear LUNs from group. See \"dmesg\" for more information.")]
    GroupClearLunFail,
    #[error("Bad attributes given for group.")]
    GroupBadAttrs,
    #[error("Group attribute '{0}' specified is static.")]
    GroupAttrStatic(String),
    #[error("Failed to set group attribute '{0}'. See \"dmesg\" for more information.")]
    GroupSetAttrFail(String),
    #[error("No such initiator '{0}' exists.")]
    GroupNoIni(String),
    #[error("Initiator '{0}' already exists.")]
    GroupIniExists(String),
    #[error("Failed to add initiator '{0}'. See \"dmesg\" for more information.")]
    GroupAddIniFail(String),
    #[error("Failed to remove initiator '{0}'. See \"dmesg\" for more information.")]
    GroupRemIniFail(String),
    #[error("Failed to move initiator '{0}'. See \"dmesg\" for more information.")]
    GroupMoveIniFail(String),
    #[error("Failed to clear initiators. See \"dmesg\" for more information.")]
    GroupClearIniFail,

    #[error("Device '{0}' already exists for LUN.")]
    LunDeviceExists(String),
    #[error("Failed to replace device '{0}' for LUN. See \"dmesg\" for more information.")]
    LunReplaceDevFail(String),
    #[error("Bad attributes for LUN.")]
    LunBadAttrs,
    #[error("Failed to set LUN attribute '{0}'. See \"dmesg\" for more information.")]
    LunAttrStatic(String),
    #[error("Failed to set LUN attribute '{0}'. See \"dmesg\" for more information.")]
    LunSetAttrFail(String),

    #[error("Bad attributes for initiator.")]
    IniBadAttrs,
    #[error("Initiator attribute '{0}' specified is static.")]
    IniAttrStatic(String),
    #[error("Failed to set initiator attribute '{0}'. See \"dmesg\" for more information.")]
    IniSetAttrFail(String),

    #[error("Session not found for driver/target.")]
    NoSession,
    #[error("Failed to close session.")]
    SessionCloseFail,
    /*

    (SCST_C_DEV_GRP_NO_GROUP)     => 'No such device group exists.',
    (SCST_C_DEV_GRP_EXISTS)       => 'Device group already exists.',
    (SCST_C_DEV_GRP_ADD_FAIL)     => 'Failed to add device group. See "dmesg" for more information.',
    (SCST_C_DEV_GRP_REM_FAIL)     => 'Failed to remove device group. See "dmesg" for more information.',

    (SCST_C_DGRP_ADD_DEV_FAIL)    => 'Failed to add device to device group. See "dmesg" for more information.',
    (SCST_C_DGRP_REM_DEV_FAIL)    => 'Failed to remove device from device group. See "dmesg" for more information.',
    (SCST_C_DGRP_NO_DEVICE)       => 'No such device in device group.',
    (SCST_C_DGRP_DEVICE_EXISTS)   => 'Device already exists within device group.',
    (SCST_C_DGRP_ADD_GRP_FAIL)    => 'Failed to add target group to device group. See "dmesg" for more information.',
    (SCST_C_DGRP_REM_GRP_FAIL)    => 'Failed to remove target group from device group. See "dmesg" for more information.',
    (SCST_C_DGRP_NO_GROUP)        => 'No such target group exists within device group.',
    (SCST_C_DGRP_GROUP_EXISTS)    => 'Target group already exists within device group.',
    (SCST_C_DGRP_DEVICE_OTHER)    => 'Device is already assigned to another device group.',

    (SCST_C_DGRP_BAD_ATTRIBUTES)   => 'Bad attributes for device group.',
    (SCST_C_DGRP_ATTRIBUTE_STATIC) => 'Device group attribute specified is static.',
    (SCST_C_DGRP_SETATTR_FAIL)     => 'Failed to set device group attribute. See "dmesg" for more information.',

    (SCST_C_TGRP_BAD_ATTRIBUTES)   => 'Bad attributes for target group.',
    (SCST_C_TGRP_ATTRIBUTE_STATIC) => 'Target group attribute specified is static.',
    (SCST_C_TGRP_SETATTR_FAIL)     => 'Failed to set target group attribute. See "dmesg" for more information.',

    (SCST_C_TGRP_ADD_TGT_FAIL)     => 'Failed to add target to target group.',
    (SCST_C_TGRP_REM_TGT_FAIL)     => 'Failed to remove target from target group.',
    (SCST_C_TGRP_NO_TGT)           => 'No such target exists within target group.',
    (SCST_C_TGRP_TGT_EXISTS)       => 'Target already exists within target group.',

    (SCST_C_TGRP_TGT_BAD_ATTR)     => 'Bad attributes for target group target.',
    (SCST_C_TGRP_TGT_ATTR_STATIC)  => 'Target group target attribute specified is static.',
    (SCST_C_TGRP_TGT_SETATTR_FAIL) => 'Failed to set target group target attribute. See "dmesg" for more information.',
         */
}

unsafe impl Sync for ScstError {}
unsafe impl Send for ScstError {}

pub trait Layer {
    fn root(&self) -> &Path;

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()>;

    fn mgmt<S: AsRef<OsStr>>(&mut self, root: S, cmd: S) -> Result<()> {
        let mgmt = Path::new(root.as_ref()).join("mgmt");
        echo(mgmt.as_ref(), cmd.as_ref())
    }
}

#[derive(Debug, Default)]
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

    pub fn contains_keys<'a>(&self, keys: &'a [String]) -> Vec<&'a String> {
        keys.iter()
            .filter(|key| self.inner.contains_key(*key))
            .collect::<Vec<&String>>()
    }

    pub fn different_set(&self, keys: &[String]) -> Vec<&String> {
        self.inner
            .iter()
            .filter_map(
                |(key, _)| {
                    if !keys.contains(key) { None } else { Some(key) }
                },
            )
            .collect::<Vec<&String>>()
    }

    pub fn to_string(&self) -> String {
        self.inner
            .iter()
            .filter_map(|(key, value)| Some(key.to_owned() + "=" + value))
            .collect::<Vec<String>>()
            .join(";")
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
    let mut fd = fs::File::open(Path::new(root.as_ref()))?;
    fd.write(cmd_str.as_bytes()).map_err(|e| ScstError::Io(e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

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

    fn foo_file(text: &str) -> Result<String> {
        let path = std::env::temp_dir().join("foo.txt");
        let mut file = fs::File::create(path.clone())?;
        file.write_all(text.as_bytes())?;
        Ok(path.to_string_lossy().to_string())
    }
}
