use anyhow::Result;
use scst::{Options, Scst, Config};

fn main() -> Result<()> {
    let mut scst = Scst::init()?;

    scst.get_handler_mut("vdisk_blockio")?.add_device(
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

    let cfg = Config::read("/tmp/scst.yml").expect("read yaml");
    scst.from_cfg(&cfg)?;

    Ok(())
}
