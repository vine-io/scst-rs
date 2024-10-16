use anyhow::Result;
use scst::{Options, Scst};

fn main() -> Result<()> {
    let mut scst = Scst::init()?;

    scst.get_handler_mut("vdisk_blockio")?.add_device(
        "vol",
        "/dev/zvol/tank/vol",
        Options::new(),
    )?;

    let target = scst
        .iscsi_mut()
        .add_target("iqn.2018-11.com.howlink:vol", Options::new())?;
    target.enable()?;
    let group = target.create_ini_group("vol")?;
    group.add_lun("vol", "0", Options::new())?;
    group.add_initiator("iqn.1988-12.com.oracle:d4ebaa45254b")?;

    let handlers = scst.handlers();
    let s = serde_yml::to_string(handlers)?;
    println!("{}", s);

    let targets = scst.iscsi().targets();
    let s = serde_yml::to_string(targets)?;
    println!("{}", s);

    let tgt = scst
        .iscsi()
        .get_target("iqn.2018-11.com.howlink:vol")
        .unwrap();
    let stat = serde_yml::to_string(&tgt.io_stat()?)?;
    println!("target stat: {}", stat);

    let sessions = serde_yml::to_string(&tgt.sessions()?)?;
    println!("target session: {}", sessions);

    Ok(())
}
