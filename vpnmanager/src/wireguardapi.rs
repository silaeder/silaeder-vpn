use std::io::{Write, prelude::*};
use std::fs::OpenOptions;
use std::process::{Command, Stdio};

static WORKINGDIR: &'static str = "/etc/wireguard";
static INTERFACE_NAME: &'static str = "wg0";

pub fn generate_keys() -> (String, String) {
    let process = Command::new("wg")
        .arg("genkey")
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let mut buffer = String::new();
    process.stdout.unwrap().read_to_string(&mut buffer).unwrap();
    let mut private_key = buffer.clone();
    private_key.pop();

    let process = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    process.stdin.unwrap().write_all(private_key.as_bytes()).unwrap();

    let mut buffer = String::new();
    process.stdout.unwrap().read_to_string(&mut buffer).unwrap();
    let mut public_key = buffer.clone();
    public_key.pop();

    (private_key, public_key)
}

pub fn dump_config(conf: String) -> () {
    let mut file = OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open(format!("{}/{}.conf", WORKINGDIR, INTERFACE_NAME))
        .unwrap();

    writeln!(file, "{}", conf);
}

pub fn sync_config() -> () {
    let process = Command::new("wg-quick")
        .arg("strip")
        .arg(INTERFACE_NAME)
        .stdout(Stdio::piped())
        .spawn().unwrap();
    
    let mut buffer = String::new();
    process.stdout.unwrap().read_to_string(&mut buffer).unwrap();
    let config = buffer.clone();
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open("storage/tmp")
        .unwrap();

    writeln!(file, "{}", config);

    let _process = Command::new("wg")
        .arg("addconf")
        .arg(INTERFACE_NAME)
        .arg("storage/tmp")
        .stdin(Stdio::piped())
        .spawn().unwrap();
}