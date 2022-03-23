use rocket::serde::Serialize;
use diesel::{self, prelude::*};

mod schema {
    table! {
        sessions {
            id -> Nullable<Integer>,
            uuid -> Text,
            token -> Text,
        }
    }
}

use self::schema::sessions;
pub use self::schema::sessions::dsl::{sessions as all_sessions};

use crate::DbConn;
use crate::utils::get_random_string;

#[derive(QueryableByName, Serialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[table_name="sessions"]
pub struct Session {
    pub id: Option<i32>,
    pub uuid: String,
    pub token: String,
}

impl Session {
    pub async fn new(uuid: String, conn: &DbConn) -> String {
        let copyofuuid = uuid.clone();
        let _res = conn.run(|c| {
            diesel::delete(all_sessions.filter(sessions::uuid.eq(copyofuuid))).execute(c)
        }).await;
        let newtoken = get_random_string(64);
        let copyoftoken = newtoken.clone();
        let _res = conn.run(|c| {
            let s = Session { 
                id: None,
                uuid: uuid,
                token: copyoftoken,
            };
            diesel::insert_into(sessions::table).values(&s).execute(c)
        }).await;
        newtoken.clone()
    }
    pub async fn end(uuid: String, token: String, conn: &DbConn) {
        let _res = conn.run(move |c| {
            diesel::delete(
                all_sessions
                .filter(sessions::uuid.eq(uuid))
                .filter(sessions::token.eq(token))
            ).execute(c)
        }).await;
    }
    pub async fn validate(uuid: String, token: String, conn: &DbConn) -> bool {
        let sessions: Vec<Session> = conn.run(move |c| {
                all_sessions
                .filter(sessions::uuid.eq(uuid))
                .filter(sessions::token.eq(token))
                .load::<Session>(c).unwrap()
        }).await;
        sessions.len() > 0
    }
}
