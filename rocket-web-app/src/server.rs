use rocket::serde::Serialize;
use diesel::{self, prelude::*};

mod schema {
    table! {
        servers {
            id -> Nullable<Integer>,
            public_key -> Text,
            private_key -> Text,
            address -> Text,
            info -> Text,
        }
    }
}

use self::schema::servers;
pub use self::schema::servers::dsl::{servers as all_servers};

use crate::DbConn;

#[derive(QueryableByName, Serialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[table_name="servers"]
pub struct Server {
    pub id: Option<i32>,
    pub public_key: String,
    pub private_key: String,
    pub address: String,
    pub info: String
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ServerPublicInfo {
    pub id: i32,
    pub info: String,
    pub address: String,
    pub public_key: String,
}

impl Server {
    pub async fn add(server_data: Server, conn: &DbConn) {
        conn.run(move |c| {
            let _res = diesel::insert_into(servers::table).values(&server_data).execute(c);
        }).await;
    }
    pub async fn remove(id: i32, conn: &DbConn) {
        let _res = conn.run(move |c| {
            diesel::delete(
                all_servers
                .filter(servers::id.eq(id))
            ).execute(c)
        }).await;
    }
    pub async fn all_servers(conn: &DbConn) -> Vec<ServerPublicInfo> {
        let servers: Vec<Server> = conn.run(|c| {
            all_servers.order(servers::id).load::<Server>(c).unwrap()
        }).await;

        let mut servers_info: Vec<ServerPublicInfo> = Vec::new();
        for server in servers {
            servers_info.push(
                ServerPublicInfo {
                    id: server.id.unwrap(),
                    info: server.info,
                    address: server.address,
                    public_key: server.public_key,
                }
            );
        }
        servers_info
    }
    pub async fn get_server(id: i32, conn: &DbConn) -> ServerPublicInfo {
        let servers: Vec<Server> = conn.run(move |c| {
            all_servers.filter(servers::id.eq(id)).load::<Server>(c).unwrap()
        }).await;
        let server = servers.iter().next().unwrap();
        ServerPublicInfo {
            id: server.id.unwrap(),
            info: server.info.to_owned(),
            address: server.address.to_owned(),
            public_key: server.public_key.to_owned(),
        }
    }
}
