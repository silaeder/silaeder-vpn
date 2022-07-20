use std::fs::OpenOptions;
use std::collections::BTreeMap;
use std::time::SystemTime;
use std::sync::Mutex;
use std::io::{Write};

use crate::wireguardapi;
use crate::CONFIG;

type DataMap = BTreeMap<u64, BTreeMap<String, (u64, u64)>>;

lazy_static::lazy_static! {
    pub static ref DATA_MAP: Mutex<DataMap> = Mutex::new(BTreeMap::new());
}

pub fn append_to_file(time: u64, data: BTreeMap<String, (u64, u64)>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&CONFIG.monitoring_data_file)
        .unwrap();

    write!(file, "{}", toml::to_string(&BTreeMap::from([(time.to_string(), data)])).unwrap());
}

pub fn append_to_map(time: u64, data: BTreeMap<String, (u64, u64)>) {
    DATA_MAP.lock().unwrap().insert(time, data);
}

pub fn get_usage_data() {
    let data = wireguardapi::get_current_stats();
    let time_stamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    append_to_map(time_stamp, data.clone());
    append_to_file(time_stamp, data.clone());
    // println!("{:#?}", DATA_MAP.lock().unwrap());
}