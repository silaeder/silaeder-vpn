use std::fs::OpenOptions;
use std::collections::BTreeMap;
use std::time::SystemTime;
use std::sync::Mutex;
use std::io::{prelude::*, Write};
use std::path::Path;

use crate::CONFIG;
use crate::servermanager::CACHE;
use crate::wireguardapi;

type DataMap = BTreeMap<u64, BTreeMap<String, (u64, u64)>>;
type CacheMap = BTreeMap<String, (u64, u64)>;

lazy_static::lazy_static! {
    pub static ref MONITORING_DATA: Mutex<DataMap> = {
        if !Path::new(&CONFIG.monitoring_data_file).exists() {
            Mutex::new(BTreeMap::new())
        } else {
            let mut buffer: String = String::new();
            let mut file = OpenOptions::new().read(true).open(&CONFIG.monitoring_data_file).unwrap();
            file.read_to_string(&mut buffer).unwrap();
            let converted: DataMap = {
                let mut tmp: DataMap = BTreeMap::new();
                for item in toml::from_str::<BTreeMap<String, BTreeMap<String, (u64, u64)>>>(&buffer).unwrap() {
                    tmp.insert(item.0.parse::<u64>().unwrap(), item.1);
                }
                tmp
            };
            Mutex::new(converted)
        }
    };
    pub static ref MONITORING_CACHE: Mutex<CacheMap> = {
        if !Path::new(&CONFIG.monitoring_cache_file).exists() {
            Mutex::new(BTreeMap::new())
        } else {
            let mut buffer: String = String::new();
            let mut file = OpenOptions::new().read(true).open(&CONFIG.monitoring_cache_file).unwrap();
            file.read_to_string(&mut buffer).unwrap();
            let deserialised: CacheMap = toml::from_str(&buffer).unwrap();
            Mutex::new(deserialised)
        }
    };
}

pub fn append_to_file(time: u64, data: BTreeMap<String, (u64, u64)>, cache: BTreeMap<String, (u64, u64)>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&CONFIG.monitoring_data_file)
        .unwrap();
    write!(file, "{}", toml::to_string(&BTreeMap::from([(time.to_string(), data)])).unwrap());
    let mut file = OpenOptions::new()
        .create(true)
        .append(false)
        .truncate(true)
        .write(true)
        .open(&CONFIG.monitoring_cache_file)
        .unwrap();
    write!(file, "{}", toml::to_string(&cache).unwrap());
}

pub fn append_to_map(time: u64, data: BTreeMap<String, (u64, u64)>) -> bool {
    if MONITORING_DATA.lock().unwrap().contains_key(&time) {
        false
    } else {
        MONITORING_DATA.lock().unwrap().insert(time, data);
        true
    }
}

pub fn get_usage_data() -> String {
    let raw_data: BTreeMap<String, (u64, u64)> = {
        let mut tmp: BTreeMap<String, (u64, u64)> = BTreeMap::new();
        for peer in wireguardapi::get_current_stats() {
            tmp.insert(CACHE.lock().unwrap().get(&peer.0).unwrap().to_string(), peer.1);
        }
        tmp
    };
    let time_stamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    for user in MONITORING_CACHE.lock().unwrap().keys() {
        if !raw_data.contains_key(user) {
            MONITORING_CACHE.lock().unwrap().insert(user.to_string(), (0, 0));
        }
    }

    let mut data: BTreeMap<String, (u64, u64)> = BTreeMap::new();

    for user in raw_data.keys().cloned() {
        if !MONITORING_CACHE.lock().unwrap().contains_key(&user) {
            MONITORING_CACHE.lock().unwrap().insert(user.to_string(), (0, 0));
        }
        let last_usage = MONITORING_CACHE.lock().unwrap()[&user].clone();
        if last_usage.0 > raw_data[&user].0 || last_usage.1 > raw_data[&user].1 {
            MONITORING_CACHE.lock().unwrap().insert(user.to_string(), (0, 0));
        }
        let new_usage = (raw_data[&user].0 - last_usage.0, raw_data[&user].1 - last_usage.1);
        MONITORING_CACHE.lock().unwrap().insert(user.to_string(), raw_data[&user]);
        data.insert(user.to_string(), new_usage);
    }

    if append_to_map(time_stamp.clone(), data.clone()) {
        append_to_file(time_stamp.clone(), data.clone(), MONITORING_CACHE.lock().unwrap().clone());
    };

    serde_json::to_string_pretty(&*MONITORING_DATA.lock().unwrap()).unwrap()
}