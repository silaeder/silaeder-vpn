use rocket::serde::Serialize;
use diesel::{self, prelude::*};

mod schema {
    table! {
        users {
            id -> Nullable<Integer>,
            name -> Text,
            custom_id -> Text,
            email -> Text,
            hashed_password -> Text,
            permission -> Integer,
            uuid -> Text,
            session -> Nullable<Integer>,
        }
    }
}

use self::schema::users;
pub use self::schema::users::dsl::{users as all_users};

use crate::DbConn;
use crate::session::Session;
use crate::utils::{hash_password, validate_password};
use uuid::Uuid;

#[derive(QueryableByName, Serialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[table_name="users"]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub custom_id: String,
    pub email: String,
    pub hashed_password: String,
    pub permission: i32,
    pub uuid: String,
    pub session: Option<i32>,
}

#[derive(Debug, FromForm)]
pub struct UserData {
    pub name: String,
    pub custom_id: String,
    pub email: String,
    pub password: String,
    pub permission: i32,
}

#[derive(Debug, FromForm)]
pub struct UserLoginData {
    pub email: String,
    pub password: String,
}


impl User {
    pub async fn add(user_data: UserData, conn: &DbConn) -> Result<String, ()> {
        let user_uuid = Uuid::new_v4().to_string();
        let user_uuid_copy = user_uuid.to_owned();
        conn.run(move |c| {
            let u = User { 
                id: None,
                name: user_data.name,
                custom_id: user_data.custom_id,
                email: user_data.email,
                hashed_password: hash_password(user_data.password),
                permission: user_data.permission,
                uuid: user_uuid_copy,
                session: None,
            };
            let _ = diesel::insert_into(users::table).values(&u).execute(c);
        }).await;
        Ok(user_uuid.clone())
    }
    pub async fn authenticate(email: String, password: String, conn: &DbConn) -> Result<(String, String), ()>  {
        let users: Vec<User> = conn.run( move |c| {
            all_users.filter(users::email.eq(email))
                .load::<User>(c).unwrap()
        }).await;
        if users.len() > 0 {
            let user = users.iter().next().unwrap();
            if validate_password(password, user.hashed_password.to_owned()){
                Ok((user.uuid.to_owned(), Session::new(user.uuid.to_owned(), conn).await))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
    pub async fn get_user(uuid: String, conn: &DbConn) -> Result<User, ()> {
        let users: Vec<User> = conn.run( move |c| {
            all_users.filter(users::uuid.eq(uuid))
                .load::<User>(c).unwrap()
        }).await;
        if users.len() > 0 {
            Ok(users[0].clone())
        } else {
            Err(())
        }
    }
}