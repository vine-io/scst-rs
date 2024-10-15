use anyhow::Result;
use scst::Scst;

fn main() -> Result<()> {
    let scst = Scst::init()?;

    let handlers = scst.handlers();
    let s = serde_yml::to_string(handlers)?;
    println!("{}", s);

    let targets = scst.iscsi().targets();
    let s = serde_yml::to_string(targets)?;
    println!("{}", s);

    let tgt = scst.iscsi().get_target("iqn.2018-11.com.howlink:vol").unwrap();
    let stat = serde_yml::to_string(&tgt.io_stat()?)?;
    println!("target stat: {}", stat);

    let sessions = serde_yml::to_string(&tgt.sessions()?)?;
    println!("target session: {}", sessions);

    Ok(())
}
