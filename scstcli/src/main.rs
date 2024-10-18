use anyhow::Result;
use scst::{Config, Options, Scst};

fn main() -> Result<()> {
    let mut scst = Scst::init().expect("init scst");

    scst.add_device(
        "vdisk_blockio",
        "vol",
        "/dev/zvol/tank/vol",
        &Options::new(),
    )?;

    let target = scst
        .iscsi_mut()
        .add_target("iqn.2018-11.com.vine:vol", &Options::new())?;
    target.enable()?;
    let group = target.create_ini_group("vol")?;
    group.add_lun("vol", 0, &Options::new())?;
    group.add_initiator("iqn.1988-12.com.oracle:d4ebaa45254b")?;

    let cfg = scst.to_cfg();
    cfg.write_to("/tmp/scst.yml")?;

    let mut scst1 = Scst::init()?;
    let target = scst1
        .iscsi_mut()
        .get_target_mut("iqn.2018-11.com.vine:vol")?;
    let group = target.get_ini_group_mut("vol")?;
    group.clear_initiators()?;
    group.del_lun(0)?;
    target.del_ini_group("vol")?;
    scst1.iscsi_mut().del_target("iqn.2018-11.com.vine:vol")?;
    scst1.del_device("vdisk_blockio", "vol")?;

    let cfg = Config::read_file("/tmp/scst.yml").expect("read yaml");
    scst1.from_cfg(&cfg)?;

    Ok(())
}
