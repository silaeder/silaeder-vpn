// password hasher

use pbkdf2::{password_hash::{rand_core::OsRng,PasswordHash, PasswordHasher, PasswordVerifier, SaltString},Pbkdf2};

pub fn hash_password(password: String) -> String {
    let password = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    Pbkdf2.hash_password::<SaltString>(password, &salt).unwrap().to_string()
}

pub fn validate_password(password: String, hash: String) -> bool {
    let password = password.as_bytes();
    let parsed_hash = PasswordHash::new(&hash).unwrap();
    Pbkdf2.verify_password(password, &parsed_hash).is_ok()
}

// token generator

use openssl::rand::rand_bytes;
use std::fmt::Write;

pub fn get_random_string(len: i32) -> String {
    let mut buf = vec![0; len as usize];
    rand_bytes(&mut buf).unwrap();
    let mut s = String::with_capacity(buf.len() * 2);
    for b in buf {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

// template context macro

#[macro_export]
macro_rules! context {
    ($($key:ident $(: $value:expr)?),*$(,)?) => {{
        use rocket::serde::ser::{Serialize, Serializer, SerializeMap};
        use ::std::fmt::{Debug, Formatter};
        use ::std::result::Result;

        #[allow(non_camel_case_types)]
        struct ContextMacroCtxObject<$($key: Serialize),*> {
            $($key: $key),*
        }

        #[allow(non_camel_case_types)]
        impl<$($key: Serialize),*> Serialize for ContextMacroCtxObject<$($key),*> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                #[allow(unused_mut)]
                let mut map = serializer.serialize_map(None)?;
                $(map.serialize_entry(stringify!($key), &self.$key)?;)*
                map.end()
            }
        }

        #[allow(non_camel_case_types)]
        impl<$($key: Debug + Serialize),*> Debug for ContextMacroCtxObject<$($key),*> {
            fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct("context!")
                    $(.field(stringify!($key), &self.$key))*
                    .finish()
            }
        }

        ContextMacroCtxObject {
            $($key $(: $value)?),*
        }
    }};
}

// user validation
use rocket::http::{ CookieJar};
use crate::DbConn;
use crate::User;
use crate::Session;

pub async fn validate(cookies: &CookieJar<'_>, conn: &DbConn) -> Result<User, ()>{
    // check if there is uuid
    let uuid_cookie = cookies.get_private("uuid");
    if !uuid_cookie.is_none() {
        let uuid = uuid_cookie.unwrap().value().to_string();
        // uuid exist => check if there is a token
        let token_cookie = cookies.get_private("token");
        if !token_cookie.is_none() {
            let token = token_cookie.unwrap().value().to_string();
            // token exitst => validate credentials
            if Session::validate(uuid.to_owned(), token.to_owned(), conn).await {
                // user authenticated => get user
                match User::get_user(uuid.to_owned(), conn).await {
                    Ok(u) => {
                        // user exist => return user
                        Ok(u)
                    },
                    Err(_) => {
                        // error getting user => exit with error
                        Err(())
                    }
                }
            } else {
                // wrong credentials => exiting with error
                Err(())
            }
        } else {
            // no token => exiting with error
            Err(())
        }
    } else {
        // no uuid => exiting with error
        Err(())
    }
}