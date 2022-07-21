// some important stuff
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

use crate::CONFIG;

pub struct Token(String);

#[derive(Debug)]
pub enum ApiTokenError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ApiTokenError;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(auth_header) => {
                let auth_str = auth_header.to_string();
                if auth_str.starts_with("Bearer") {
                    let token = auth_str[6..auth_str.len()].trim();
                    if token == CONFIG.auth_token {
                        request::Outcome::Success(Token(token.to_string()))
                    } else {
                        request::Outcome::Failure((Status::Unauthorized, ApiTokenError::Invalid))
                    }
                } else {
                    request::Outcome::Failure((Status::Unauthorized, ApiTokenError::Invalid))
                }
            }
            None => request::Outcome::Failure((Status::Unauthorized, ApiTokenError::Missing)),
        }
    }
}

// Some debug and info controls
#[get("/")]
pub fn index() -> String {
    "VPN backend is running".to_string()
}

// Wireguard controls
pub mod wg {
    use crate::servermanager;
    use crate::wireguardapi;
    use rocket::response::content;
    use rocket::State;
    use std::sync::Mutex;
    
    #[get("/generate_keys")]
    pub fn generate_keys(_token: crate::webinterface::Token) -> content::Json<String> {
        let key_tuple = wireguardapi::generate_keys();
        content::Json(serde_json::to_string_pretty(&key_tuple).unwrap())
    }
    
    
    #[post("/restart")]
    pub fn restart(_token: crate::webinterface::Token) -> () {
        wireguardapi::restart()
    }
    
    #[post("/dump_config")]
    pub fn dump_config(
        server: &State<Mutex<servermanager::Server>>,
        _token: crate::webinterface::Token,
    ) -> () {
        wireguardapi::dump_config(server.lock().unwrap().get_server_config())
    }
    
    // Don't Use Temporary deprecated
    #[post("/sync_config")]
    pub fn sync_config(_token: crate::webinterface::Token) -> () {
        wireguardapi::sync_config()
    }
}

// Server data management
pub mod server {
    use crate::servermanager;
    use rocket::response::content;
    use rocket::State;
    use std::sync::Mutex;
    use crate::CONFIG;
    
    #[post("/new_peer/<info>")]
    pub fn new_peer(
        server: &State<Mutex<servermanager::Server>>,
        info: String,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().new_peer(info)
    }
    
    #[get("/get_server_config")]
    pub fn get_server_config(
        server: &State<Mutex<servermanager::Server>>,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().get_server_config()
    }

    #[get("/get_client_config/<id>")]
    pub fn get_client_config_by_id(
        server: &State<Mutex<servermanager::Server>>,
        id: usize,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().get_client_config_by_id(id)
    }

    #[get("/get_client_config/<info>", rank=2)]
    pub fn get_client_config_by_info(
        server: &State<Mutex<servermanager::Server>>,
        info: String,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().get_client_config_by_info(info)
    }

    #[get("/dump_to_json")]
    pub fn dump_to_json(
        server: &State<Mutex<servermanager::Server>>,
        _token: crate::webinterface::Token,
    ) -> content::Json<String> {
        content::Json(server.lock().unwrap().dump_to_json())
    }

    #[post("/dump_to_file")]
    pub fn dump_to_file(
        server: &State<Mutex<servermanager::Server>>,
        _token: crate::webinterface::Token,
    ) -> () {
        server
            .lock()
            .unwrap()
            .dump_to_file(CONFIG.server_dump_file.to_string())
    }

    #[post("/load_from_file")]
    pub fn load_from_file(
        server: &State<Mutex<servermanager::Server>>,
        _token: crate::webinterface::Token,
    ) -> () {
        server
            .lock()
            .unwrap()
            .load_from_file(CONFIG.server_dump_file.to_string())
    }
}

pub mod monitoring {
    use rocket::response::content;

    use crate::monitoring;

    use crate::servermanager::CACHE;

    #[get("/get_stats")]
    pub fn get_stats(_token: crate::webinterface::Token) -> content::Json<String> {
        content::Json(monitoring::get_usage_data())
    }
}
// #[options("/<_..>")]
// pub fn send_options() {

// }