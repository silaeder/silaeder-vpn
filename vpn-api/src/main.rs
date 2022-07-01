#[macro_use] extern crate rocket;

mod servermanager;
mod webinterface;
mod wireguardapi;

use std::sync::Mutex;

#[rocket::main]
async fn main() {

    let res = wireguardapi::generate_keys();
    let s = Mutex::new(servermanager::Server::new(
        String::from("1303"),
        String::from("enp37s0"),
        res.1,
        res.0,
        String::from("justdprroz.ru"),
    ));

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
        // .attach(CORS)
        .launch()
        .await;
}