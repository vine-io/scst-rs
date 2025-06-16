use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    IOStat, Layer, Options, ScstError, Session, cmd_with_options, echo, read_dir, read_fl,
    read_link, read_stat,
};

static TARGET_GROUP: &str = "ini_groups";
static TARGET_LUN: &str = "luns";
static TARGET_INITIATOR: &str = "initiators";
static TARGET_SESSION: &str = "sessions";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Driver {
    #[serde(skip)]
    root: String,
    name: String,
    enabled: i8,
    open_state: String,
    version: String,

    targets: BTreeMap<String, Target>,
}

impl Driver {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled == 1
    }

    pub(crate) fn enabled_i8(&self) -> i8 {
        self.enabled
    }

    /// enable scst driver
    pub fn enable(&mut self) -> Result<()> {
        let root = self.root().join("enabled");
        let cmd = "1";
        echo(root, cmd.into())?;

        self.enabled = 1;
        Ok(())
    }

    /// disable scst driver
    pub fn disable(&mut self) -> Result<()> {
        let root = self.root().join("enabled");
        let cmd = "0";
        echo(root, cmd.into())?;

        self.enabled = 0;

        Ok(())
    }

    pub fn open_state(&self) -> &str {
        &self.open_state
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn targets(&self) -> Vec<&Target> {
        self.targets.values().collect()
    }

    pub fn get_target<S: AsRef<str>>(&self, name: S) -> Result<&Target> {
        self.targets
            .get(name.as_ref())
            .context(ScstError::NoTarget(name.as_ref().to_string()))
    }

    pub fn get_target_mut<S: AsRef<str>>(&mut self, name: S) -> Result<&mut Target> {
        self.targets
            .get_mut(name.as_ref())
            .context(ScstError::NoTarget(name.as_ref().to_string()))
    }

    /// create a scst target, like 'iqn.2018-11.com.vine:test'
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let mut options = Options::new();
    /// scst.iscsi_mut().add_target("iqn.2018-11.com.vine:test", &options)?;
    /// ```
    pub fn add_target<S: AsRef<str>>(&mut self, name: S, options: &Options) -> Result<&mut Target> {
        let name_ref = name.as_ref();
        if self.targets.contains_key(name_ref) {
            anyhow::bail!(ScstError::TargetExists(name_ref.to_string()))
        }

        let root = self.root();
        let mut cmd = format!("add_target {}", name_ref);
        let params = vec![
            "IncomingUser".to_string(),
            "OutgoingUser".to_string(),
            "allowed_portal".to_string(),
        ];
        cmd = options
            .pack_with_check(&params)?
            .and_then(|s| {
                let mut c = cmd.clone();
                c.push_str(" ");
                c.push_str(&s);
                Some(c)
            })
            .or(Some(cmd))
            .unwrap();

        self.mgmt(root.to_path_buf(), cmd.into())?;

        let mut target = Target::default();
        target.load(self.root().join(name_ref))?;
        self.targets.insert(target.name().to_string(), target);

        self.get_target_mut(name_ref)
    }

    /// delete a scst target, like 'iqn.2018-11.com.vine:test'
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let mut options = Options::new();
    /// scst.iscsi_mut().del_target("iqn.2018-11.com.vine:test")?;
    /// ```
    pub fn del_target<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        let name_ref = name.as_ref();
        if !self.targets.contains_key(name_ref) {
            anyhow::bail!(ScstError::NoTarget(name_ref.to_string()))
        }

        let root = self.root();
        let cmd = format!("del_target {}", name_ref);
        self.mgmt(root.to_path_buf(), cmd.into())?;

        self.targets.remove(name_ref);

        Ok(())
    }

    pub fn add_target_attribute<S: AsRef<str>>(
        &mut self,
        name: S,
        attr: S,
        value: S,
    ) -> Result<()> {
        let name_ref = name.as_ref();
        if !self.targets.contains_key(name_ref) {
            anyhow::bail!(ScstError::NoTarget(name_ref.to_string()))
        }

        let root = self.root();
        let cmd = format!(
            "add_target_attribute {} {} {}",
            name_ref,
            attr.as_ref(),
            value.as_ref()
        );
        let params = vec![
            "IncomingUser".to_string(),
            "OutgoingUser".to_string(),
            "allowed_portal".to_string(),
        ];

        if !params.contains(&attr.as_ref().to_string()) {
            anyhow::bail!(ScstError::TargetBadAttrs)
        }

        self.mgmt(root.to_path_buf(), cmd.into())?;

        let mut target = Target::default();
        target.load(self.root().join(name_ref))?;
        self.targets.insert(target.name().to_string(), target);

        Ok(())
    }

    pub fn del_target_attribute<S: AsRef<str>>(
        &mut self,
        name: S,
        attr: S,
        value: S,
    ) -> Result<()> {
        let name_ref = name.as_ref();
        if !self.targets.contains_key(name_ref) {
            anyhow::bail!(ScstError::NoTarget(name_ref.to_string()))
        }

        let root = self.root();
        let cmd = format!(
            "del_target_attribute {} {} {}",
            name_ref,
            attr.as_ref(),
            value.as_ref()
        );
        let params = vec![
            "IncomingUser".to_string(),
            "OutgoingUser".to_string(),
            "allowed_portal".to_string(),
        ];

        if !params.contains(&attr.as_ref().to_string()) {
            anyhow::bail!(ScstError::TargetBadAttrs)
        }

        self.mgmt(root.to_path_buf(), cmd.into())?;

        let mut target = Target::default();
        target.load(self.root().join(name_ref))?;
        self.targets.insert(target.name().to_string(), target);

        Ok(())
    }

    pub fn add_attribute<S: AsRef<str>>(&mut self, attr: S, value: S) -> Result<()> {
        let root = self.root();
        let cmd = format!("add_attribute {} {}", attr.as_ref(), value.as_ref());
        let params = vec!["IncomingUser".to_string(), "OutgoingUser".to_string()];

        if !params.contains(&attr.as_ref().to_string()) {
            anyhow::bail!(ScstError::TargetBadAttrs)
        }

        self.mgmt(root.to_path_buf(), cmd.into())?;

        Ok(())
    }

    pub fn del_attribute<S: AsRef<str>>(&mut self, attr: S, value: S) -> Result<()> {
        let root = self.root();
        let cmd = format!("del_attribute {} {}", attr.as_ref(), value.as_ref());
        let params = vec!["IncomingUser".to_string(), "OutgoingUser".to_string()];

        if !params.contains(&attr.as_ref().to_string()) {
            anyhow::bail!(ScstError::TargetBadAttrs)
        }

        self.mgmt(root.to_path_buf(), cmd.into())?;

        Ok(())
    }
}

impl Layer for Driver {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P>(&mut self, root: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.name = root_ref
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();
        self.enabled = read_fl(root_ref.join("enabled"))?.parse::<i8>()?;
        self.open_state = read_fl(root_ref.join("open_state"))?;
        self.version = read_fl(root_ref.join("version"))?;

        // traverse target directory
        self.targets = read_dir(root_ref)?
            .filter_map(|res| res.ok())
            .filter(|entry| {
                entry.path().is_dir() && entry.file_name().to_string_lossy().starts_with("iqn")
            })
            .filter_map(|entry| {
                let mut target = Target::default();
                target.set_name(entry.file_name().to_string_lossy());
                target.load(entry.path()).ok();
                Some((target.name().to_string(), target))
            })
            .collect();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Target {
    #[serde(skip)]
    root: String,
    tid: u64,
    rel_tgt_id: u64,
    name: String,
    enabled: i8,

    luns: BTreeMap<String, Lun>,
    ini_groups: BTreeMap<String, IniGroup>,
}

impl Target {
    pub fn tid(&self) -> u64 {
        self.tid
    }

    pub fn rel_tgt_id(&self) -> u64 {
        self.rel_tgt_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn set_name<S: AsRef<str>>(&mut self, name: S) {
        self.name = name.as_ref().to_string()
    }

    /// get scst target state
    pub fn enabled(&self) -> bool {
        self.enabled == 1
    }

    pub(crate) fn enabled_i8(&self) -> i8 {
        self.enabled
    }

    /// enable scst target
    pub fn enable(&mut self) -> Result<()> {
        let root = self.root().join("enabled");
        let cmd = "1";
        echo(root, cmd.into())?;

        self.enabled = 1;
        Ok(())
    }

    /// disable scst target
    pub fn disable(&mut self) -> Result<()> {
        let root = self.root().join("enabled");
        let cmd = "0";
        echo(root, cmd.into())?;

        self.enabled = 0;

        Ok(())
    }

    pub fn luns(&self) -> Vec<&Lun> {
        self.luns.values().collect()
    }

    pub fn get_lun<S: AsRef<str>>(&self, lun_id: S) -> Result<&Lun> {
        self.luns
            .get(lun_id.as_ref())
            .context(ScstError::TargetNoLun(lun_id.as_ref().to_string()))
    }

    pub fn get_lun_mut<S: AsRef<str>>(&mut self, lun_id: S) -> Result<&mut Lun> {
        self.luns
            .get_mut(lun_id.as_ref())
            .context(ScstError::TargetNoLun(lun_id.as_ref().to_string()))
    }

    /// create a lun for target.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// target.add_lun("disk1", 0, &Options::new())?;
    /// ```
    pub fn add_lun<S: AsRef<str>>(
        &mut self,
        device: S,
        lun_id: u64,
        options: &Options,
    ) -> Result<()> {
        let id_ref = lun_id.to_string();
        if self.luns.contains_key(&format!("lun{}", &id_ref)) {
            anyhow::bail!(ScstError::TargetLunExists(id_ref.clone()))
        }

        let mut cmd = format!("add {} {}", device.as_ref(), &id_ref);
        let params = vec!["read_only".to_string()];
        cmd = cmd_with_options(&cmd, &params, &options)?;

        let root = self.root().join(TARGET_LUN);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::TargetAddLunFail(id_ref.clone()))?;

        let mut lun = Lun::default();
        lun.load(self.root().join(TARGET_LUN).join(&id_ref))?;
        self.luns.insert(lun.name().to_string(), lun);

        Ok(())
    }

    pub fn set_lun<S: AsRef<str>>(
        &mut self,
        device: S,
        lun_id: u64,
        options: &Options,
    ) -> Result<()> {
        let id_ref = lun_id.to_string();
        let name = format!("lun{}", &id_ref);
        if !self.luns.contains_key(&name) {
            anyhow::bail!(ScstError::TargetNoLun(id_ref.clone()))
        }

        let mut cmd = format!("replace {} {}", device.as_ref(), &id_ref);
        let params = vec!["read_only".to_string()];
        cmd = cmd_with_options(&cmd, &params, &options)?;

        let root = self.root().join(TARGET_LUN);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::LunSetAttrFail(id_ref.clone()))?;

        let mut lun = Lun::default();
        lun.load(self.root().join(TARGET_LUN).join(&id_ref))?;
        self.luns.insert(lun.name().to_string(), lun);

        Ok(())
    }

    /// delete a lun for target.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// target.del_lun(0)?;
    /// ```
    pub fn del_lun(&mut self, lun_id: u64) -> Result<()> {
        let id_ref = lun_id.to_string();
        let name = format!("lun{}", &id_ref);
        if !self.luns.contains_key(&name) {
            anyhow::bail!(ScstError::TargetNoLun(id_ref.clone()))
        }

        let root = self.root().join(TARGET_LUN);
        let cmd = format!("del {}", &id_ref);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::TargetRemLunFail(id_ref.clone()))?;

        self.luns.remove(&name);

        Ok(())
    }

    pub fn ini_groups(&self) -> Vec<&IniGroup> {
        self.ini_groups.values().collect()
    }

    pub fn get_ini_group<S: AsRef<str>>(&self, name: S) -> Result<&IniGroup> {
        self.ini_groups
            .get(name.as_ref())
            .context(ScstError::NoGroup(name.as_ref().to_string()))
    }

    pub fn get_ini_group_mut<S: AsRef<str>>(&mut self, name: S) -> Result<&mut IniGroup> {
        self.ini_groups
            .get_mut(name.as_ref())
            .context(ScstError::NoGroup(name.as_ref().to_string()))
    }

    /// create a initiator group for target.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// target.create_ini_group("test")?;
    /// ```
    pub fn create_ini_group<S: AsRef<str>>(&mut self, name: S) -> Result<&mut IniGroup> {
        let name_ref = name.as_ref();
        if self.ini_groups.contains_key(name_ref) {
            anyhow::bail!(ScstError::GroupExists(name_ref.to_string()))
        }

        let root = self.root().join(TARGET_GROUP);
        let cmd = format!("create {}", name_ref);
        self.mgmt(root, cmd.into())?;

        let mut group = IniGroup::default();
        group.load(self.root().join(TARGET_GROUP).join(name_ref))?;
        self.ini_groups.insert(group.name().to_string(), group);

        self.get_ini_group_mut(name)
    }

    /// delete a initiator group for target.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// target.del_ini_group("test")?;
    /// ```
    pub fn del_ini_group<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        let name_ref = name.as_ref();
        if !self.ini_groups.contains_key(name_ref) {
            anyhow::bail!(ScstError::NoGroup(name_ref.to_string()))
        }

        let root = self.root().join(TARGET_GROUP);
        let cmd = format!("del {}", name_ref);
        self.mgmt(root, cmd.into())?;

        self.ini_groups.remove(name_ref);

        Ok(())
    }

    pub fn io_stat(&self) -> Result<IOStat> {
        read_stat(self.root())
    }

    pub fn sessions(&self) -> Result<Vec<Session>> {
        let sessions = read_dir(self.root().join(TARGET_SESSION))?
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                let mut session = Session::default();
                session.load(entry.path()).ok();
                Some(session)
            })
            .collect();

        Ok(sessions)
    }
}

impl Layer for Target {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.name = root_ref
            .file_name()
            .and_then(|s| Some(s.to_string_lossy().to_string()))
            .or(Some("".to_string()))
            .unwrap();
        self.tid = read_fl(root_ref.join("tid"))
            .unwrap_or("0".to_string())
            .parse::<u64>()?;
        self.rel_tgt_id = read_fl(root_ref.join("rel_tgt_id"))?.parse::<u64>()?;
        self.enabled = read_fl(root_ref.join("enabled"))
            .unwrap_or("1".to_string())
            .parse::<i8>()?;

        // traverse target luns
        self.luns = read_dir(root_ref.join(TARGET_LUN))?
            .filter_map(|res| res.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let mut lun = Lun::default();
                lun.load(entry.path()).ok();
                Some((lun.name().to_string(), lun))
            })
            .collect();

        // traverse target groups
        self.ini_groups = read_dir(root_ref.join(TARGET_GROUP))?
            .filter_map(|res| res.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let mut ini_group = IniGroup::default();
                ini_group.load(entry.path()).ok();
                Some((ini_group.name().to_string(), ini_group))
            })
            .collect();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IniGroup {
    #[serde(skip)]
    root: String,
    name: String,

    luns: BTreeMap<String, Lun>,
    initiators: Vec<String>,
}

impl IniGroup {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn luns(&self) -> Vec<&Lun> {
        self.luns.values().collect()
    }

    pub fn get_lun<S: AsRef<str>>(&self, lun_id: S) -> Result<&Lun> {
        self.luns
            .get(lun_id.as_ref())
            .context(ScstError::GroupNoLun(lun_id.as_ref().to_string()))
    }

    pub fn get_lun_mut<S: AsRef<str>>(&mut self, lun_id: S) -> Result<&mut Lun> {
        self.luns
            .get_mut(lun_id.as_ref())
            .context(ScstError::GroupNoLun(lun_id.as_ref().to_string()))
    }

    /// create a lun for target initiator group.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// let group = target.get_ini_group("test")?;
    /// group.add_lun("disk1", 0, &Options::new())?;
    /// ```
    pub fn add_lun<S: AsRef<str>>(
        &mut self,
        device: S,
        lun_id: u64,
        options: &Options,
    ) -> Result<()> {
        let id_ref = lun_id.to_string();
        let name = format!("lun{}", &id_ref);
        if self.luns.contains_key(&name) {
            anyhow::bail!(ScstError::GroupLunExists(id_ref.clone()))
        }

        let mut cmd = format!("add {} {}", device.as_ref(), &id_ref);
        let params = vec!["read_only".to_string()];
        cmd = cmd_with_options(&cmd, &params, &options)?;

        let root = self.root().join(TARGET_LUN);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::GroupAddLunFail(id_ref.clone()))?;

        let mut lun = Lun::default();
        lun.load(self.root().join(TARGET_LUN).join(&id_ref))?;
        self.luns.insert(lun.name().to_string(), lun);

        Ok(())
    }

    pub fn set_lun<S: AsRef<str>>(
        &mut self,
        device: S,
        lun_id: u64,
        options: &Options,
    ) -> Result<()> {
        let id_ref = lun_id.to_string();
        let name = format!("lun{}", &id_ref);
        if !self.luns.contains_key(&name) {
            anyhow::bail!(ScstError::GroupNoLun(id_ref.clone()))
        }

        let mut cmd = format!("replace {} {}", device.as_ref(), &id_ref);
        let params = vec!["read_only".to_string()];
        cmd = cmd_with_options(&cmd, &params, &options)?;

        let root = self.root().join(TARGET_LUN);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::LunSetAttrFail(id_ref.clone()))?;

        let mut lun = Lun::default();
        lun.load(self.root().join(TARGET_LUN).join(&id_ref))?;
        self.luns.insert(lun.name().to_string(), lun);

        Ok(())
    }

    /// delete a lun for target initiator group.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// let group = target.get_ini_group("test")?;
    /// group.del_lun(0)?;
    /// ```
    pub fn del_lun(&mut self, lun_id: u64) -> Result<()> {
        let id_ref = lun_id.to_string();
        let name = format!("lun{}", &id_ref);
        if !self.luns.contains_key(&name) {
            anyhow::bail!(ScstError::GroupNoLun(id_ref.clone()))
        }

        let root = self.root().join(TARGET_LUN);
        let cmd = format!("del {}", id_ref);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::GroupRemLunFail(id_ref.clone()))?;

        self.luns.remove(&name);

        Ok(())
    }

    pub fn initiators(&self) -> &[String] {
        &self.initiators
    }

    /// add an initiator for target initiator group.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// let group = target.get_ini_group("test")?;
    /// group.add_initiator("iqn.1988-12.com.oracle:d4ebaa45254")?;
    /// ```
    pub fn add_initiator<S: AsRef<str>>(&mut self, initiator: S) -> Result<()> {
        let ini = initiator.as_ref();
        if self.initiators.contains(&ini.to_string()) {
            anyhow::bail!(ScstError::GroupIniExists(ini.to_string()))
        }

        let root = self.root().join(TARGET_INITIATOR);
        let cmd = format!("add {}", ini);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::GroupAddIniFail(ini.to_string()))?;

        self.initiators.push(ini.to_string());

        Ok(())
    }

    /// del an initiator for target initiator group.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// let group = target.get_ini_group("test")?;
    /// group.del_initiator("iqn.1988-12.com.oracle:d4ebaa45254")?;
    /// ```
    pub fn del_initiator<S: AsRef<str>>(&mut self, initiator: S) -> Result<()> {
        let ini = initiator.as_ref();
        if !self.initiators.contains(&ini.to_string()) {
            anyhow::bail!(ScstError::GroupNoIni(ini.to_string()))
        }

        let root = self.root().join(TARGET_INITIATOR);
        let cmd = format!("del {}", ini);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::GroupRemIniFail(ini.to_string()))?;

        if let Some(index) = self.initiators.iter().position(|item| *item == ini) {
            self.initiators.remove(index);
        }

        Ok(())
    }

    /// move the initiator to other group.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// let group = target.get_ini_group("test")?;
    /// group.move_initiator("test1", "iqn.1988-12.com.oracle:d4ebaa45254")?;
    /// ```
    pub fn move_initiator<S: AsRef<str>>(&mut self, initiator: S, dest_group: S) -> Result<()> {
        let ini = initiator.as_ref().to_string();
        let group = dest_group.as_ref();
        if !self.initiators.contains(&ini) {
            anyhow::bail!(ScstError::GroupNoIni(ini))
        }

        let root = self.root().join(TARGET_INITIATOR);
        let cmd = format!("move {} {}", ini, group);
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::GroupMoveIniFail(ini))?;

        Ok(())
    }

    /// clear all initiators to initiator group.
    ///
    /// ```no_run
    /// use scst::{Scst, Options}
    ///
    /// let mut scst = Scst::init()?;
    ///
    /// let target = scst.iscsi_mut().get_target_mut("iqn.2018-11.com.vine:test")?;
    /// let mut group = target.get_ini_group("test")?;
    /// group.clear_initiators()?;
    /// ```
    pub fn clear_initiators(&mut self) -> Result<()> {
        let root: std::path::PathBuf = self.root().join(TARGET_INITIATOR);
        let cmd = "clear";
        self.mgmt(root, cmd.into())
            .map_err(|_| ScstError::GroupClearIniFail)?;

        Ok(())
    }
}

impl Layer for IniGroup {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.name = root_ref
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();

        // traverse group luns
        self.luns = read_dir(root_ref.join(TARGET_LUN))?
            .filter_map(|res| res.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let mut lun = Lun::default();
                lun.load(entry.path()).ok();
                Some((lun.name().to_string(), lun))
            })
            .collect();

        // traverse group initiators
        self.initiators = read_dir(root_ref.join(TARGET_INITIATOR))?
            .filter_map(|res| res.ok())
            .filter(|e| e.path().is_file() && e.file_name().to_string_lossy().starts_with("iqn"))
            .filter_map(|e| Some(e.file_name().to_string_lossy().to_string()))
            .collect::<Vec<String>>();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Lun {
    #[serde(skip)]
    root: String,
    id: u64,
    device: String,
    read_only: i8,
}

impl Lun {
    pub fn name(&self) -> String {
        "lun".to_string() + &self.id.to_string()
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn read_only(&self) -> bool {
        self.read_only == 1
    }
}

impl Layer for Lun {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.id = root_ref
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string()
            .parse::<u64>()?;
        self.device = read_link(root_ref.join("device"))?
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();
        self.read_only = read_fl(root_ref.join("read_only"))?.parse::<i8>()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use regex::Regex;

    #[test]
    fn read_ips() -> Result<()> {
        let re = Regex::new(r"^(?:\d{1,3}\.){3}\d{1,3}$")?;
        assert!(re.is_match("192.168.2.30"));

        Ok(())
    }
}
