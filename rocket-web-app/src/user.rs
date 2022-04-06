use rocket::serde::{Serialize, Deserialize};
use diesel::{self, prelude::*};

mod schema {
    table! {
        users {
            id -> Nullable<Integer>,
            name -> Text,
            username -> Text,
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

#[derive(QueryableByName, Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[table_name="users"]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub permission: i32,
    pub uuid: String,
    pub session: Option<i32>,
}

#[derive(Debug, FromForm)]
pub struct UserData {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub permission: i32,
}

#[derive(Debug, FromForm)]
pub struct UserLoginData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, FromForm, Clone)]
pub struct UserQuery {
    pub name: String,
    pub username: String,
    pub email: String,
    pub uuid: String,
}

#[derive(Debug, FromForm, Clone)]
pub struct UserPasswordMod {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, FromForm, Clone)]
pub struct UserUsernameMod {
    pub new_username: String,
}

#[derive(Debug, FromForm, Clone)]
pub struct UserDeleteForm {
    pub uuid: String,
}

impl User {
    pub async fn get_all(conn: &DbConn) -> Vec<User> {
        let users: Vec<User> = conn.run(move |c| {
            all_users.load::<User>(c).unwrap()
        }).await;
        users
    }
    pub async fn add(user_data: UserData, conn: &DbConn) -> Result<String, ()> {
        let user_uuid = Uuid::new_v4().to_string();
        let user_uuid_copy = user_uuid.to_owned();
        conn.run(move |c| {
            let u = User { 
                id: None,
                name: user_data.name,
                username: user_data.username,
                email: user_data.email.to_lowercase(),
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
            all_users.filter(users::email.eq(email.to_lowercase()))
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

    pub async fn get_user_by_uuid(uuid: String, conn: &DbConn) -> Result<User, ()> {
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

    pub async fn delete_user_by_uuid(uuid: String, conn: &DbConn) {
        conn.run( move |c| {
            diesel::delete(all_users.filter(users::uuid.eq(uuid)))
                .execute(c).unwrap();
        }).await;
    }

    pub async fn get_users_by_query(query: UserQuery, conn: &DbConn) -> Vec<User> {
        let users: Vec<User> = conn.run( move |c| {
            let mut res = all_users.order(users::id).into_boxed();
            if query.name != "" {
                res = res.filter(users::name.like(format!("%{}%", query.name)));
            }
            if query.username != "" {
                res = res.filter(users::username.like(format!("%{}%", query.username)));
            }
            if query.email != "" {
                res = res.filter(users::email.like(format!("%{}%", query.email)));
            }
            if query.uuid != "" {
                res = res.filter(users::uuid.like(format!("%{}%", query.uuid)));
            }
            res.load::<User>(c).unwrap()
        }).await;
        users
    }

    pub async fn change_password(uid: String, np: String, conn: &DbConn) {
        let _res = conn.run( move |c| {
            diesel::update(all_users.filter(users::uuid.eq(uid)))
            .set(users::hashed_password.eq(hash_password(np))).execute(c)
        }).await;
    }

    pub async fn change_username(uid: String, nun: String, conn: &DbConn) {
        let _res = conn.run( move |c| {
            diesel::update(all_users.filter(users::uuid.eq(uid)))
            .set(users::username.eq(nun)).execute(c)
        }).await;
    }
}