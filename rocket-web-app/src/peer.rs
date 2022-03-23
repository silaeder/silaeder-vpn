use rocket::serde::Serialize;
use diesel::{self, prelude::*};

mod schema {
    table! {
        peers {
            id -> Nullable<Integer>,
            public_key -> Text,
            private_key -> Text,
            address -> Text,
            server_public_key -> Text,
            server_address -> Text,
            owner_uuid -> Text,
            owner_name -> Text,
        }
    }
}

use self::schema::peers;
pub use self::schema::peers::dsl::{peers as all_peers};

use crate::DbConn;

#[derive(QueryableByName, Serialize, Queryable, Insertable, Debug, Clone, FromForm)]
#[serde(crate = "rocket::serde")]
#[table_name="peers"]
pub struct Peer {
    pub id: Option<i32>,
    pub public_key: String,
    pub private_key: String,
    pub address: String,
    pub server_public_key: String,
    pub server_address: String,
    pub owner_uuid: String,
    pub owner_name: String,
}

impl Peer {
    pub async fn add(peer_data: Peer, conn: &DbConn) {
        conn.run(move |c| {
            let _res = diesel::insert_into(peers::table).values(&peer_data).execute(c);
        }).await
    }
    pub async fn remove(id: i32, conn: &DbConn) {
        let _res = conn.run(move |c| {
            diesel::delete(
                all_peers
                .filter(peers::id.eq(id))
            ).execute(c)
        }).await;
    }
    pub async fn get_all_for_user(owner: String, conn: &DbConn) -> Vec<Peer> {
        let peers: Vec<Peer> = conn.run(move |c| {
            all_peers.filter(peers::owner_uuid.eq(owner))
                .load::<Peer>(c).unwrap()
        }).await;
        peers
    }
    pub async fn get_peer(id: i32, conn: &DbConn) -> Peer {
        let peers: Vec<Peer> = conn.run(move |c| {
            all_peers.filter(peers::id.eq(id)).load::<Peer>(c).unwrap()
        }).await;
        peers[0].clone()
    }
}
