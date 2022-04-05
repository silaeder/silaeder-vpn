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
use crate::user::{User, UserData, UserLoginData, UserQuery};
use crate::session::Session;
use crate::peer::{Peer, PeerQuery};
use std::fs::File;
use std::path::Path;
use std::io::Write;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[post("/add_user", data = "<userdata>")]
async fn useradd(cookies: &CookieJar<'_>, userdata: Form<UserData>, conn: DbConn) -> Result<Flash<Redirect>, Redirect> {
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                let user_uuid = User::add(userdata.into_inner(), &conn).await.unwrap();
                return Ok(Flash::new(Redirect::to("/admin"), "added_user_uuid", user_uuid.to_owned()))
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
        Err(_) => {
            Err(Flash::error(Redirect::to("/login"), "Неправильный логин или пароль"))
        }
    }
}

#[post("/logout")]
async fn userlogout(cookies: &CookieJar<'_>, conn: DbConn) -> Redirect {
    match validate(cookies, &conn).await {
        Ok(_) => {
            let uuid = cookies.get_private("uuid").unwrap().value().to_owned();
            let token = cookies.get_private("token").unwrap().value().to_owned();
            cookies.remove_private(cookies.get_private("uuid").unwrap());
            cookies.remove_private(cookies.get_private("token").unwrap());
            Session::end(uuid, token, &conn).await;
        },
        Err(_) => {},
    }
    Redirect::to("/login")
}

#[post("/add_peer", data = "<peerdata>")]
async fn add_peer(cookies: &CookieJar<'_>, peerdata: Form<Peer>, conn: DbConn) -> Redirect {
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                Peer::add(peerdata.into_inner(), &conn).await;
                Redirect::to("/admin")
            } else {
                Redirect::to("/dashboard")
            }
        },
        Err(_) => {
            Redirect::to("/login")
        },
    }
}

#[post("/search_user", data = "<userq>")]
async fn searchuser(cookies: &CookieJar<'_>, userq: Form<UserQuery>, conn: DbConn) -> Result<Flash<Redirect>, ()> {
    let userquery = userq.into_inner();
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                Ok(Flash::new(Redirect::to("/admin"), "users", serde_json::to_string(&User::get_users_by_query(userquery, &conn).await).unwrap()))
            } else {
                Err(())
            }
        },
        Err(_) => {
            Err(())
        },
    }
}

#[post("/delete_user", data = "<userq>")]
async fn deleteuser(cookies: &CookieJar<'_>, userq: Form<UserQuery>, conn: DbConn) -> Result<Flash<Redirect>, ()> {
    let userquery = userq.into_inner();
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                Ok(Flash::new(Redirect::to("/admin"), "users", serde_json::to_string(&User::delete_users_by_query(userquery, &conn).await).unwrap()))
            } else {
                Err(())
            }
        },
        Err(_) => {
            Err(())
        },
    }
}

#[post("/search_peer", data = "<peerq>")]
async fn searchpeer(cookies: &CookieJar<'_>, peerq: Form<PeerQuery>, conn: DbConn) -> Result<Flash<Redirect>, ()> {
    let peerquery = peerq.into_inner();
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                Ok(Flash::new(Redirect::to("/admin"), "peers", serde_json::to_string(&Peer::get_peers_by_query(peerquery, &conn).await).unwrap()))
            } else {
                Err(())
            }
        },
        Err(_) => {
            Err(())
        },
    }
}

#[post("/delete_peer", data = "<peerq>")]
async fn deletepeer(cookies: &CookieJar<'_>, peerq: Form<PeerQuery>, conn: DbConn) -> Result<Flash<Redirect>, ()> {
    let peerquery = peerq.into_inner();
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                Ok(Flash::new(Redirect::to("/admin"), "peers", serde_json::to_string(&Peer::delete_peers_by_query(peerquery, &conn).await).unwrap()))
            } else {
                Err(())
            }
        },
        Err(_) => {
            Err(())
        },
    }
}

#[get("/config/<id>")]
async fn get_config(cookies: &CookieJar<'_>, id: i32, conn: DbConn) -> Result<NamedFile, Redirect> {
    match Peer::get_peer(id, &conn).await {
        Ok(p) => {
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
        },
        Err(_) => {
            Err(Redirect::to("/dashboard"))
        }
    }
    
}

#[get("/allusers")]
async fn get_all_user(cookies: &CookieJar<'_>, conn: DbConn) -> Result<String, Redirect> {
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                Ok(format!("{:#?}", User::get_all(&conn).await))
            } else {
                Err(Redirect::to("/dashboard"))
            }
        },
        Err(_) => {
            Err(Redirect::to("/login"))
        }
    }
}

#[get("/")]
async fn index() -> Redirect {
    Redirect::to("/dashboard")
}

#[get("/dashboard")]
async fn dashboard(cookies: &CookieJar<'_>, conn: DbConn) -> Result<Template, Redirect> {
    match validate(cookies, &conn).await {
        Ok(u) => {
            println!("{:#?}", context!{peers: Peer::get_all_for_user(u.uuid.clone(), &conn).await, username: u.name.clone()});
            Ok(Template::render("dashboard", context! {peers: Peer::get_all_for_user(u.uuid.clone(), &conn).await, username: u.name.clone()}))
        },
        Err(_) => {
            Err(Redirect::to("/login"))
        }
    }
}

#[get("/admin")]
async fn admin(flash: Option<FlashMessage<'_>>, cookies: &CookieJar<'_>, conn:DbConn) -> Result<Template, Redirect> {
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10{
                let flash = flash.map(FlashMessage::into_inner);
                match flash {
                    Some(f) => {
                        if f.0 == "users" {
                            println!("{}", f.1.clone());
                            let deser: Vec<User> = serde_json::from_str(f.1.clone().as_str()).unwrap();
                            Ok(Template::render("admin", context!{flash: (f.0.clone(), deser)}))
                        } else if f.0 == "peers" {
                            println!("{}", f.1.clone());
                            let deser: Vec<Peer> = serde_json::from_str(f.1.clone().as_str()).unwrap();
                            Ok(Template::render("admin", context!{flash: (f.0.clone(), deser)}))
                        } else {
                            Ok(Template::render("admin", context!{flash: (f.0.clone(), f.1.clone())}))
                        }
                    },
                    None => {
                        Ok(Template::render("admin", context!{}))
                    },
                }
            } else {
                Err(Redirect::to("/dashboard"))
            }
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
            if u.permission >= 10 {
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
        .mount("/", routes![index, dashboard, contact, login, request_peer, get_config, add_peer, get_all_user, admin, searchuser, deleteuser, searchpeer, deletepeer])
        .mount("/auth", routes![userlogin, useradd, userlogout])
}