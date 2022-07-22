#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

mod servermanager;
mod webinterface;
mod wireguardapi;
mod monitoring;

use std::sync::Mutex;
use std::fs;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    server_port: String,
    ip_interface_name: String,
    public_address: String,
    auth_token: String,
    server_dump_file: String,
    subnet: i64,
    wg_workingdir: String,
    wg_interface_name: String,
    monitoring_data_file: String,
    monitoring_cache_file: String
}

lazy_static! {
    pub static ref CONFIG: Config = toml::from_str(&fs::read_to_string("VPN.toml").unwrap()).unwrap();
}

#[rocket::main]
async fn main() {
    let s: Mutex<servermanager::Server>;

    if !Path::new(&CONFIG.server_dump_file).exists() {
        let res = wireguardapi::generate_keys();
        s = Mutex::new(servermanager::Server::new(
            String::from(&CONFIG.server_port),
            String::from(&CONFIG.ip_interface_name),
            res.1,
            res.0,
            String::from(&CONFIG.public_address),
        ));
    } else {
        s = Mutex::new(servermanager::Server::empty());
        s
            .lock()
            .unwrap()
            .load_from_file(CONFIG.server_dump_file.to_string());
    }

    wireguardapi::dump_config(s.lock().unwrap().get_server_config());
    wireguardapi::restart();

    let _ = rocket::build()
        .manage(s)
        .mount(
            // api status page
            "/api",
            routes![
                webinterface::index,
                // webinterface::send_options
            ]
        )
        .mount(
            // wireguard api
            "/api/wg",
            routes![
                webinterface::wg::generate_keys,
                webinterface::wg::dump_config,
                webinterface::wg::sync_config,
                webinterface::wg::restart
            ],
        )
        .mount(
            // server info manager
            "/api/manage",
            routes![
                webinterface::server::new_peer,
                webinterface::server::get_server_config,
                webinterface::server::get_client_config_by_id,
                webinterface::server::get_client_config_by_info,
                webinterface::server::dump_to_json,
                webinterface::server::dump_to_file,
                webinterface::server::load_from_file
            ],
        )
        .mount(
            "/api/data",
            routes![
                webinterface::monitoring::update_stats
            ]
        )
        // .attach(CORS)
        .launch()
        .await;
}