use std::path::Path;

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{Layer, read_dir, read_fl};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IOStat {
    bidi_cmd_count: usize,
    bidi_io_count_kb: usize,
    bidi_unaligned_cmd_count: usize,

    write_cmd_count: usize,
    write_io_count_kb: usize,
    write_unaligned_cmd_count: usize,

    read_cmd_count: usize,
    read_io_count_kb: usize,
    read_unaligned_cmd_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Session {
    #[serde(skip)]
    root: String,
    sid: String,
    thread_pid: String,
    initiator_name: String,

    ips: Vec<SessionIP>,
}

impl Session {
    pub fn sid(&self) -> &str {
        &self.sid
    }

    pub fn thread_pid(&self) -> &str {
        &self.thread_pid
    }

    pub fn initiator_name(&self) -> &str {
        &self.initiator_name
    }

    pub fn ips(&self) -> Vec<&SessionIP> {
        self.ips.iter().collect()
    }

    pub fn io_stat(&self) -> Result<IOStat> {
        read_stat(self.root())
    }
}

impl Layer for Session {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.sid = read_fl(root_ref.join("sid"))?;
        self.thread_pid = read_fl(root_ref.join("thread_pid"))?;
        self.initiator_name = read_fl(root_ref.join("initiator_name"))?;

        let ip_re = Regex::new(r"^(?:\d{1,3}\.){3}\d{1,3}$")?;
        self.ips = read_dir(root_ref)?
            .filter_map(|res| res.ok())
            .filter(|entry| {
                entry.path().is_dir() && ip_re.is_match(&entry.file_name().to_string_lossy())
            })
            .filter_map(|entry| {
                let mut ip = SessionIP::default();
                ip.load(entry.path()).ok();
                Some(ip)
            })
            .collect();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SessionIP {
    #[serde(skip)]
    root: String,
    cid: String,
    ip: String,
    state: String,
    target_ip: String,
}

impl SessionIP {
    pub fn cid(&self) -> &str {
        &self.cid
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn target_ip(&self) -> &str {
        &self.target_ip
    }
}

impl Layer for SessionIP {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.cid = read_fl(root_ref.join("cid"))?;
        self.ip = read_fl(root_ref.join("ip"))?;
        self.state = read_fl(root_ref.join("state"))?;
        self.target_ip = read_fl(root_ref.join("target_ip"))?;

        Ok(())
    }
}

pub fn read_stat<S: AsRef<Path>>(root: S) -> Result<IOStat> {
    let root_ref = root.as_ref();
    let bidi_cmd_count = read_fl(root_ref.join("bidi_cmd_count"))?.parse::<usize>()?;
    let bidi_io_count_kb = read_fl(root_ref.join("bidi_io_count_kb"))?.parse::<usize>()?;
    let bidi_unaligned_cmd_count =
        read_fl(root_ref.join("bidi_unaligned_cmd_count"))?.parse::<usize>()?;
    let write_cmd_count = read_fl(root_ref.join("write_cmd_count"))?.parse::<usize>()?;
    let write_io_count_kb = read_fl(root_ref.join("write_io_count_kb"))?.parse::<usize>()?;
    let write_unaligned_cmd_count =
        read_fl(root_ref.join("write_unaligned_cmd_count"))?.parse::<usize>()?;
    let read_cmd_count = read_fl(root_ref.join("read_cmd_count"))?.parse::<usize>()?;
    let read_io_count_kb = read_fl(root_ref.join("read_io_count_kb"))?.parse::<usize>()?;
    let read_unaligned_cmd_count =
        read_fl(root_ref.join("read_unaligned_cmd_count"))?.parse::<usize>()?;

    let stat = IOStat {
        bidi_cmd_count,
        bidi_io_count_kb,
        bidi_unaligned_cmd_count,
        write_cmd_count,
        write_io_count_kb,
        write_unaligned_cmd_count,
        read_cmd_count,
        read_io_count_kb,
        read_unaligned_cmd_count,
    };

    Ok(stat)
}
