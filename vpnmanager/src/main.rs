
mod wireguardapi;
mod servermanager;

use wireguardapi::{generate_keys, dump_config, sync_config};
use servermanager::{Server};

fn main() {
    let res = generate_keys();
    let mut s: Server = Server::new(res.0, res.1, String::from("justdprroz.ru"));

    let mut ids: Vec<u64> = Vec::new();

    for _i in 0..253 {
        ids.push(s.new_peer());
    }
    
    s.dump_to_json("storage/server_dump.json".to_string());

    dump_config(s.get_server_config("55000".to_string(), "enp6s0".to_string()));
    sync_config();
}
