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
    use std::sync::Arc;
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
        server: &State<Arc<Mutex<servermanager::Server>>>,
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
    use crate::CONFIG;
    use rocket::response::content;
    use rocket::State;
    use std::sync::Arc;
    use std::sync::Mutex;

    #[post("/new_peer/<info>")]
    pub fn new_peer_info(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        info: String,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().new_peer(info)
    }
    #[post("/new_peer", rank = 2)]
    pub fn new_peer_no_info(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().new_peer("".to_string())
    }

    #[get("/get_server_config")]
    pub fn get_server_config(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().get_server_config()
    }

    #[get("/get_client_config/<id>")]
    pub fn get_client_config_by_id(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        id: usize,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().get_client_config_by_id(id)
    }

    #[get("/get_client_config/<info>", rank = 2)]
    pub fn get_client_config_by_info(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        info: String,
        _token: crate::webinterface::Token,
    ) -> String {
        server.lock().unwrap().get_client_config_by_info(info)
    }

    #[get("/dump_to_json")]
    pub fn dump_to_json(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        _token: crate::webinterface::Token,
    ) -> content::Json<String> {
        content::Json(server.lock().unwrap().dump_to_json())
    }

    #[post("/dump_to_file")]
    pub fn dump_to_file(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        _token: crate::webinterface::Token,
    ) -> () {
        server
            .lock()
            .unwrap()
            .dump_to_file(CONFIG.server_dump_file.to_string())
    }

    #[post("/load_from_file")]
    pub fn load_from_file(
        server: &State<Arc<Mutex<servermanager::Server>>>,
        _token: crate::webinterface::Token,
    ) -> () {
        server
            .lock()
            .unwrap()
            .load_from_file(CONFIG.server_dump_file.to_string())
    }
}

pub mod monitoring {
    use crate::monitoring;
    use rocket::response::content;
    use std::time::SystemTime;

    #[post("/update_stats")]
    pub fn update_stats(_token: crate::webinterface::Token) -> () {
        monitoring::update_usage_data();
    }

    #[get("/get_stats/<step>?<last>")]
    pub fn get_stats_last(
        _token: crate::webinterface::Token,
        step: &str,
        last: &str,
    ) -> content::Json<String> {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        get_stats_period(
            _token,
            step,
            {
                match last {
                    "hour" => current_time - 3600,
                    "day" => current_time - 86400,
                    "week" => current_time - 604800,
                    _ => current_time,
                }
            }
            .to_string(),
            current_time.to_string(),
        )
    }

    #[get("/get_stats/<step>?<from>&<to>", rank = 2)]
    pub fn get_stats_period(
        _token: crate::webinterface::Token,
        step: &str,
        from: String,
        to: String,
    ) -> content::Json<String> {
        content::Json(
            serde_json::to_string_pretty(&monitoring::get_usage(
                from.parse::<u64>().unwrap(),
                to.parse::<u64>().unwrap(),
                match step {
                    "minute" => 60,
                    "hour" => 3600,
                    "day" => 86400,
                    "week" => 604800,
                    _ => 60,
                },
            ))
            .unwrap(),
        )
    }
}
// #[options("/<_..>")]
// pub fn send_options() {

// }
