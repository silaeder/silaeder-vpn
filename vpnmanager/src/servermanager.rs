use crate::wireguardapi;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{prelude::*, Write};

#[derive(Debug)]
pub struct Interface {
    private_key: String,
    address: String,
    listen_port: Option<String>,
    dns: Option<String>,
    post_up: Option<String>,
    post_down: Option<String>,
}

#[derive(Debug)]
pub struct Peer {
    public_key: String,
    allowed_ips: String,
    endpoint: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Server {
    port: String,
    nic: String,
    public_key: String,
    private_key: String,
    address: String,
    clients: Vec<Client>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Client {
    _id: u64,
    public_key: String,
    private_key: String,
    address: String,
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result: String = String::from("[Interface]\n");

        result.push_str(format!("PrivateKey = {}\n", self.private_key).as_str());
        result.push_str(format!("Address = {}\n", self.address).as_str());
        if !self.listen_port.is_none() {
            result
                .push_str(format!("ListenPort = {}\n", self.listen_port.clone().unwrap()).as_str());
        }
        if !self.dns.is_none() {
            result.push_str(format!("DNS = {}\n", self.dns.clone().unwrap()).as_str());
        }
        if !self.post_up.is_none() {
            result.push_str(format!("PostUp = {}\n", self.post_up.clone().unwrap()).as_str());
        }
        if !self.post_down.is_none() {
            result.push_str(format!("PostDown = {}\n", self.post_down.clone().unwrap()).as_str());
        }

        write!(f, "{}", result)
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result: String = String::from("[Peer]\n");
        result.push_str(format!("PublicKey = {}\n", self.public_key).as_str());
        result.push_str(format!("AllowedIPs = {}\n", self.allowed_ips).as_str());
        if !self.endpoint.is_none() {
            result.push_str(format!("Endpoint = {}\n", self.endpoint.as_ref().unwrap()).as_str());
        }

        write!(f, "{}", result)
    }
}

impl Server {
    pub fn new(
        port: String,
        nic: String,
        public_key: String,
        private_key: String,
        address: String,
    ) -> Server {
        Server {
            port: port,
            nic: nic,
            public_key: public_key,
            private_key: private_key,
            address: address,
            clients: Vec::new(),
        }
    }

    pub fn new_peer(&mut self) -> u64 {
        let key_pair = wireguardapi::generate_keys();
        let c = Client {
            _id: self.clients.len() as u64,
            public_key: key_pair.0,
            private_key: key_pair.1,
            address: format!("10.0.0.{}", 2 + self.clients.len() as u64),
        };
        self.clients.push(c);
        (self.clients.len() - 1) as u64
    }

    pub fn get_server_config(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.as_interface().to_string());
        result.push_str("\n");
        for client in &self.clients {
            result.push_str(&client.as_peer().to_string());
            result.push_str("\n");
        }
        result
    }

    pub fn get_client_config(&self, client_id: u64) -> String {
        let mut result = self.clients[client_id as usize].as_interface().to_string();
        result.push_str("\n");
        result.push_str(&self.as_peer().to_string());
        result.push_str("\n");
        result
    }

    pub fn dump_to_json(&self) -> String {
        let serialised = serde_json::to_string_pretty(self).unwrap();
        serialised
    }

    pub fn dump_to_file(&self, path: String) -> () {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .unwrap();
        writeln!(file, "{}", &self.dump_to_json()).unwrap();
    }

    pub fn load_from_file(&mut self, path: String) -> () {
        let mut buffer: String = String::new();
        let mut file = OpenOptions::new().read(true).open(path).unwrap();

        file.read_to_string(&mut buffer).unwrap();
        let deserialised: Server = serde_json::from_str(&buffer).unwrap();
        *self = deserialised
    }

    pub fn as_interface(&self) -> Interface {
        Interface {
            private_key: self.private_key.clone(),
            address: "10.0.0.1/32".to_string(),
            listen_port: Some(self.port.clone()),
            post_up: Some(format!("iptables -A FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o {} -j MASQUERADE", self.nic)),
            post_down: Some(format!("iptables -D FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o {} -j MASQUERADE", self.nic)),
            dns: None,
        }
    }

    pub fn as_peer(&self) -> Peer {
        Peer {
            public_key: self.public_key.clone(),
            allowed_ips: "0.0.0.0/0".to_string(),
            endpoint: Some(self.address.clone()),
        }
    }
}

impl Client {
    pub fn as_peer(&self) -> Peer {
        Peer {
            public_key: self.public_key.clone(),
            allowed_ips: format!("{}/32", self.address),
            endpoint: None,
        }
    }
    pub fn as_interface(&self) -> Interface {
        Interface {
            private_key: self.private_key.clone(),
            address: format!("{}/32", self.address),
            dns: Some("8.8.8.8".to_string()),
            listen_port: None,
            post_up: None,
            post_down: None,
        }
    }
}
