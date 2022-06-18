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
mod servermanager;
mod webinterface;
mod wireguardapi;

use rocket::{Request, Response};
use rocket::{Rocket, Build};
use rocket::fairing::{Fairing, Info, Kind, AdHoc};
use rocket::response::{Flash, Redirect};
use rocket::fs::NamedFile;
use rocket::request::FlashMessage;
use rocket::form::Form;
use rocket::fs::{FileServer, relative};
use rocket::post;
use rocket::http::{Cookie, CookieJar, Header};
use rocket_dyn_templates::{Template};

use crate::utils::{validate, validate_password};
use crate::user::{User, UserData, UserLoginData, UserQuery, UserPasswordMod, UserUsernameMod, UserDeleteForm};
use crate::session::Session;
use crate::peer::{Peer, PeerQuery};
use std::fs::{self, File};
use std::path::Path;
use std::io::Write;
use std::sync::Mutex;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "http://192.168.1.4:3000"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

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
async fn deleteuser(cookies: &CookieJar<'_>, userq: Form<UserDeleteForm>, conn: DbConn) -> Result<Redirect, ()> {
    let userquery = userq.into_inner();
    match validate(cookies, &conn).await {
        Ok(u) => {
            if u.permission >= 10 {
                User::delete_user_by_uuid(userquery.uuid, &conn).await;
                Ok(Redirect::to("/admin"))
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

#[post("/change_password", data = "<passwordform>")]
async fn changepassword(cookies: &CookieJar<'_>, passwordform: Form<UserPasswordMod>, conn: DbConn) -> Redirect {
    let passwordupdate = passwordform.into_inner();
    match validate(cookies, &conn).await {
        Ok(u) => {
            if validate_password(passwordupdate.current_password.clone(), u.hashed_password.clone()) {
                User::change_password(u.uuid, passwordupdate.new_password, &conn).await;
            }
            Redirect::to("/settings")
        },
        Err(_) => {
            Redirect::to("/login")
        },
    }
}

#[post("/change_username", data = "<usernameform>")]
async fn changeusername(cookies: &CookieJar<'_>, usernameform: Form<UserUsernameMod>, conn: DbConn) -> Redirect {
    let usernameupdate = usernameform.into_inner();
    match validate(cookies, &conn).await {
        Ok(u) => {
            User::change_username(u.uuid, usernameupdate.new_username, &conn).await;
            Redirect::to("/settings")
        },
        Err(_) => {
            Redirect::to("/login")
        },
    }
}

#[get("/config/<id>/<_filename>")]
async fn get_config(cookies: &CookieJar<'_>, id: i32, _filename: String, conn: DbConn) -> Result<NamedFile, Redirect> {
    match Peer::get_peer(id, &conn).await {
        Ok(p) => {
            match validate(cookies, &conn).await {
                Ok(u) => {
                    if u.uuid == p.owner_uuid {
                        let sp = format!("cache_files/{}_{}.conf", u.username, p.public_key[1..5].to_string());
                        let mut file = File::create(Path::new(&sp)).unwrap();
                        let config = format!(
                            "[Interface]\n\
                            PrivateKey = {}\n\
                            Address = {}\n\
                            DNS = 1.1.1.1, 8.8.8.8\n\
                            \n\
                            [Peer]\n\
                            PublicKey = {}\n\
                            AllowedIPs = 0.0.0.0/0\n\
                            Endpoint = {}\n\
                            PersistentKeepalive = 20",
                            p.private_key, p.address, p.server_public_key, p.server_address);
                        let _res = write!(&mut file, "{}", config);
                        let nf = NamedFile::open(sp.clone()).await.ok().unwrap();
                        let _res = fs::remove_file(sp.clone());
                        Ok(nf)
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
            Ok(Template::render("dashboard",
                context! {
                    peers: Peer::get_all_for_user(u.uuid.clone(), &conn).await,
                    name: u.name.clone(),
                    username: u.username.clone(),
                    admin: u.permission >= 10
                }
            ))
        },
        Err(_) => {
            Err(Redirect::to("/login"))
        }
    }
}

#[get("/settings")]
async fn settings(cookies: &CookieJar<'_>, conn: DbConn) -> Result<Template, Redirect> {
    match validate(cookies, &conn).await {
        Ok(u) => {
            Ok(Template::render("settings",
                context! {
                    name: u.name.clone(),
                    username: u.username.clone()
                }
            ))
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
                            Ok(Template::render("admin",
                                context! {
                                    flash: (f.0.clone(), deser),
                                    name: u.name.clone()
                                }
                            ))
                        } else if f.0 == "peers" {
                            println!("{}", f.1.clone());
                            let deser: Vec<Peer> = serde_json::from_str(f.1.clone().as_str()).unwrap();
                            Ok(Template::render("admin",
                                context!{
                                    flash: (f.0.clone(), deser),
                                    name: u.name.clone()
                                }
                            ))
                        } else {
                            Ok(Template::render("admin",
                                context!{
                                    flash: (f.0.clone(), f.1.clone()),
                                    name: u.name.clone()
                                }
                            ))
                        }
                    },
                    None => {
                        Ok(Template::render("admin", 
                            context!{
                                name: u.name.clone()
                            }
                        ))
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

#[get("/contact")]
async fn contact(cookies: &CookieJar<'_>, conn:DbConn) -> Template {
    match validate(cookies, &conn).await {
        Ok(u) => {
            Template::render("contact",
                context!{
                    name: u.name.clone()
                }
            )
        },
        Err(_) => {
            Template::render("contact",
                context!{}
            )
        }
    }
}

#[get("/login")]
async fn login(cookies: &CookieJar<'_>, flash: Option<FlashMessage<'_>>, conn:DbConn) -> Result<Redirect, Template> {
    match validate(cookies, &conn).await {
        Ok(_) => {
            Ok(Redirect::to("/dashboard"))
        },
        Err(_) => {
            let flash = flash.map(FlashMessage::into_inner);
            match flash {
                Some(f) => {
                    Err(Template::render("login", context!{flash: (f.0, f.1)}))
                },
                None => {
                    Err(Template::render("login", context!{}))
                },
            }
        },
    }
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!();
    let conn = DbConn::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c)).await.expect("can run migrations");
    rocket
}

#[rocket::main]
async fn main() {

    let res = wireguardapi::generate_keys();
    let s = Mutex::new(servermanager::Server::new(
        String::from("1303"),
        String::from("enp37s0"),
        res.1,
        res.0,
        String::from("justdprroz.ru"),
    ));

    let _ = rocket::build()
        .manage(s)
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .mount(
            // serve static js and css
            "/",
            FileServer::from(relative!("static"))
        )
        .mount(
            // html render
            "/",
            routes![index, dashboard, contact, login, admin, settings]
        )
        .mount(
            // general api for management 
            "/", 
            routes![get_config, add_peer, get_all_user, searchuser, deleteuser, searchpeer, deletepeer]
        )
        .mount(
            // auth api 
            "/auth", 
            routes![userlogin, useradd, userlogout, changepassword, changeusername]
        )
        .mount(
            // api status page
            "/api",
            routes![
                webinterface::index,
                webinterface::send_options
            ]
        )
        .mount(
            // wireguard api
            "/api/wg",
            routes![
                webinterface::wg::generate_keys,
                webinterface::wg::dump_config,
                webinterface::wg::sync_config,
            ],
        )
        .mount(
            // server info manager
            "/api/manage",
            routes![
                webinterface::server::new_peer,
                webinterface::server::get_server_config,
                webinterface::server::get_client_config,
                webinterface::server::dump_to_json,
                webinterface::server::dump_to_file,
                webinterface::server::load_from_file
            ],
        )
        .attach(CORS)
        .launch()
        .await;
}