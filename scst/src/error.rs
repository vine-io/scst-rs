use thiserror::Error;

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
    #[error("Failed to add device '{name}': '{e}'. See \"dmesg\" for more information.")]
    DeviceAddFail { name: String, e: anyhow::Error },
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
