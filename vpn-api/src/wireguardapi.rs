use std::fs::OpenOptions;
use std::io::{prelude::*, Write};
use std::process::{Command, Stdio};

use crate::CONFIG;

pub fn generate_keys() -> (String, String) {
    let process = Command::new("wg")
        .arg("genkey")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut buffer = String::new();
    process.stdout.unwrap().read_to_string(&mut buffer).unwrap();
    let mut private_key = buffer.clone();
    private_key.pop();

    let process = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    process
        .stdin
        .unwrap()
        .write_all(private_key.as_bytes())
        .unwrap();

    let mut buffer = String::new();
    process.stdout.unwrap().read_to_string(&mut buffer).unwrap();
    let mut public_key = buffer.clone();
    public_key.pop();

    (private_key, public_key)
}

pub fn dump_config(conf: String) -> () {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{}/{}.conf", CONFIG.wg_workingdir, CONFIG.wg_interface_name))
        .unwrap();

    writeln!(file, "{}", conf).unwrap();
}

// TODO: FIXME: Sync don't work properly
pub fn sync_config() -> () {
    let process = Command::new("wg-quick")
        .arg("strip")
        .arg(CONFIG.wg_interface_name.to_string())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut buffer = String::new();
    process.stdout.unwrap().read_to_string(&mut buffer).unwrap();
    let config = buffer.clone();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("storage/tmp")
        .unwrap();

    writeln!(file, "{}", config).unwrap();

    let _process = Command::new("wg")
        .arg("addconf")
        .arg(CONFIG.wg_interface_name.to_string())
        .arg("storage/tmp")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
}

pub fn restart() -> () {
    let _process = Command::new("wg-quick")
        .arg("down")
        .arg(CONFIG.wg_interface_name.to_string())
        .spawn()
        .unwrap();

    let _process = Command::new("wg-quick")
        .arg("up")
        .arg(CONFIG.wg_interface_name.to_string())
        .spawn()
        .unwrap();
}