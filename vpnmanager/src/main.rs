#[macro_use]
extern crate rocket;

mod servermanager;
mod webinterface;
mod wireguardapi;
use std::sync::Mutex;

#[rocket::main]
async fn main() {
    let res = wireguardapi::generate_keys();
    let s = Mutex::new(servermanager::Server::new(
        String::from("55000"),
        String::from("enp6s0"),
        res.0,
        res.1,
        String::from("justdprroz.ru"),
    ));
    let _ = rocket::build()
        .manage(s)
        .mount("/", routes![webinterface::index])
        .mount(
            "/wg",
            routes![
                webinterface::wg::generate_keys,
                webinterface::wg::dump_config,
                webinterface::wg::sync_config
            ],
        )
        .mount(
            "/manage",
            routes![
                webinterface::server::new_peer,
                webinterface::server::get_server_config,
                webinterface::server::get_client_config,
                webinterface::server::dump_to_json,
                webinterface::server::dump_to_file,
                webinterface::server::load_from_file
            ],
        )
        .launch()
        .await;
}
