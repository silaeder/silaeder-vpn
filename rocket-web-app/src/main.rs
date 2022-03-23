#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket_sync_db_pools;

//<-----------------------JUST_IMPORTS----------------------------------------->

mod utils;
mod session;
mod user;
mod server;
mod peer;

use rocket::{Rocket, Build};
use rocket::fairing::AdHoc;
use rocket::response::{Flash, Redirect};
use rocket::fs::NamedFile;
use rocket::request::FlashMessage;
use rocket::form::Form;
use rocket::fs::{FileServer, relative};
use rocket::post;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::{Template};

use crate::utils::validate;
use crate::user::{User, UserData, UserLoginData};
use crate::session::Session;
use crate::peer::Peer;
use std::fs::File;
use std::path::Path;
use std::io::Write;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

//<-----------------------API_REQUESTS----------------------------------------->

#[post("/add_user", data = "<userdata>")]
async fn useradd(cookies: &CookieJar<'_>, userdata: Form<UserData>, conn: DbConn) -> Result<String, Redirect> {
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission > 5 {
                let user_uuid = User::add(userdata.into_inner(), &conn).await.unwrap();
                return Ok(user_uuid.to_owned())
            }
        },
        Err(_) => {
        },
    }
    Err(Redirect::to("/"))
}

#[post("/login", data = "<userdata>")]
async fn userlogin(cookies: &CookieJar<'_>, userdata: Form<UserLoginData>, conn: DbConn) -> Result<Redirect, Flash<Redirect>> {
    match User::authenticate(userdata.email.clone(), userdata.password.clone(), &conn).await {
        Ok((uuid, token)) => {
            cookies.add_private(Cookie::new("uuid", uuid));
            cookies.add_private(Cookie::new("token", token));
            Ok(Redirect::to("/"))
        },
        _ => {
            Err(Flash::error(Redirect::to("/login"), "Неправильный логин или пароль"))
        }
    }
}

#[post("/logout")]
async fn userlogout(cookies: &CookieJar<'_>, conn: DbConn) -> Redirect {    
    let mut token: String = "".to_owned();
    let token_cookie = cookies.get_private("token");
    if !token_cookie.is_none() {
        let token_cookie = token_cookie.unwrap();
        token = token_cookie.value().to_string();
        cookies.remove_private(token_cookie);
    }
    let mut uuid: String = "".to_owned();
    let uuid_cookie = cookies.get_private("uuid");
    if !uuid_cookie.is_none() {
        let uuid_cookie = uuid_cookie.unwrap();
        uuid = uuid_cookie.value().to_string();
        cookies.remove_private(uuid_cookie);
    }
    Session::end(uuid.to_owned(), token.to_owned(), &conn).await;
    Redirect::to("/login")
}

#[get("/config/<id>")]
async fn get_config(cookies: &CookieJar<'_>, id: i32, conn: DbConn) -> Result<NamedFile, Redirect> {
    let p = Peer::get_peer(id, &conn).await;
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.uuid == p.owner_uuid {
                let sp = format!("cache_files/{}_{}.conf", p.owner_name, p.id.unwrap());
                {
                    let mut file = File::create(Path::new(&sp)).unwrap();
                    writeln!(&mut file, "[Interface]");
                    writeln!(&mut file, "PrivateKey = {}", p.private_key);
                    writeln!(&mut file, "Address = {}", p.address);
                    writeln!(&mut file, "DNS = 1.1.1.1, 8.8.8.8");
                    writeln!(&mut file, "");
                    writeln!(&mut file, "[Peer]");
                    writeln!(&mut file, "PublicKey = {}", p.server_public_key);
                    writeln!(&mut file, "AllowedIPs = 0.0.0.0/0");
                    writeln!(&mut file, "Endpoint = {}", p.server_address);
                    writeln!(&mut file, "PersistentKeepalive = 20");
                }
                Ok(NamedFile::open(sp).await.ok().unwrap())
            } else {
                Err(Redirect::to("/dashboard"))
            }
        },
        Err(_) => {
            Err(Redirect::to("/login"))
        }
    }
}
//<-----------------------RENDER_REQUESTS-------------------------------------->

#[get("/")]
async fn index() -> Redirect {
    Redirect::to("/dashboard")
}

#[get("/dashboard")]
async fn dashboard(cookies: &CookieJar<'_>, conn: DbConn) -> Result<Template, Redirect> {
    match validate(cookies, &conn).await {
        Ok(u) => {
            Ok(Template::render("dashboard", context! {peers: Peer::get_all_for_user(u.uuid, &conn).await, username: u.name}))
        },
        Err(_) => {
            Err(Redirect::to("/login"))
        }
    }
}

#[get("/request")]
async fn request_peer(cookies: &CookieJar<'_>, conn: DbConn) -> Redirect {
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission > 5 {
                Peer::add(
                    Peer {
                        id: None,
                        public_key: "public".to_string(),
                        private_key: "private_key".to_string(),
                        address: "10.0.0.1".to_string(),
                        server_public_key: "new pub key".to_string(),
                        server_address: "1.32.42.5".to_string(),
                        owner_uuid: u.uuid.to_owned(),
                        owner_name: u.name.to_string(),
                    }, &conn
                ).await;
            }
        },
        Err(_) => {},
    }
    Redirect::to("/")
}

#[get("/contact")]
async fn contact() -> Template {
    Template::render("contact", context!{})
}
#[get("/login")]
fn login(flash: Option<FlashMessage<>>) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    match flash {
        Some(f) => {
            Template::render("login", context!{flash: (f.0, f.1)})
        },
        None => {
            Template::render("login", context!{})
        },
    }
}



//<-----------------------BACKEND_MAGIC---------------------------------------->

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!();
    let conn = DbConn::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c)).await.expect("can run migrations");
    rocket
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![index, dashboard, contact, login, request_peer, get_config])
        .mount("/auth", routes![userlogin, useradd, userlogout])
}