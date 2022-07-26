use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{prelude::*, Write};
use std::process::{Command, Stdio};

use crate::CONFIG;

pub fn generate_keys() -> (String, String) {
    println!("Wireguard: Generating keypair");
    let mut process = Command::new("wg")
        .arg("genkey")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut buffer = String::new();
    process
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut buffer)
        .unwrap();
    process.wait().unwrap();
    let mut private_key = buffer.clone();
    private_key.pop();

    let mut process = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    process
        .stdin
        .take()
        .unwrap()
        .write_all(private_key.as_bytes())
        .unwrap();

    let mut buffer = String::new();
    process
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut buffer)
        .unwrap();
    process.wait().unwrap();
    let mut public_key = buffer.clone();
    public_key.pop();

    (private_key, public_key)
}

pub fn dump_config(conf: String) -> () {
    println!("Wireguard: Dumping server config to WireGuard config");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!(
            "{}/{}.conf",
            CONFIG.wg_workingdir, CONFIG.wg_interface_name
        ))
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
    println!("Wireguard: Stopping WireGuard");
    let _process = Command::new("wg-quick")
        .arg("down")
        .arg(CONFIG.wg_interface_name.to_string())
        .spawn()
        .unwrap()
        .wait();

    println!("Wireguard: Starting Wireguard");
    let _process = Command::new("wg-quick")
        .arg("up")
        .arg(CONFIG.wg_interface_name.to_string())
        .spawn()
        .unwrap()
        .wait();
}

pub fn get_current_stats() -> BTreeMap<String, (u64, u64)> {
    println!("Wireguard: Getting data");

    let mut process = Command::new("wg")
        .arg("show")
        .arg(CONFIG.wg_interface_name.to_owned())
        .arg("transfer")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut buffer = String::new();
    process
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut buffer)
        .unwrap();
    process.wait().unwrap();
    buffer.pop();

    let mut data: BTreeMap<String, (u64, u64)> = BTreeMap::new();

    for peer in buffer.split('\n') {
        let parts: Vec<&str> = peer.split("\t").collect();
        data.insert(
            parts[0].to_string(),
            (parts[1].parse().unwrap(), parts[2].parse().unwrap()),
        );
    }

    data
}
