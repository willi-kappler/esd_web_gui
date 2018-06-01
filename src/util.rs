use std::collections::HashMap;
use std::sync::Mutex;
use std::convert::From;

use serde::{Serialize};
use handlebars::{Handlebars};
use argon2;

use error::{WebGuiError};
use failure;

lazy_static! {
    static ref TEMPLATE : Handlebars = {
        let mut hb = Handlebars::new();
        hb.register_template_file("login", "html/login.hbs").unwrap();
        hb.register_template_file("logout", "html/logout.hbs").unwrap();
        hb.register_template_file("menu", "html/menu.hbs").unwrap();
        hb
    };

    static ref USERDATA: Mutex<HashMap<String, UserData>> = {
        Mutex::new(HashMap::new())
    };
}

struct UserData {
    login_id: String,
}

fn get_hash_from_db(login_id: &str) -> Result<String, failure::Error> {
    info!("util.rs, get_hash_from_db()");
    // TODO!!!
    Ok("$argon2i$v=19$m=4096,t=3,p=1$cm9oYmF1Y2hhYzlUdW8wY2k2UmF1bmd1aGFpZzVzb2hjb29Ob2hjaXdlcmVlczRiYWtlZXRoM0NvaGJpZUxhaA$KAta8FGbVMSv/OsA/PGL0FXrNfjJ4Gv6SUkaiZKYbHA".to_string())
}

pub fn logged_in(client_id: &str) -> Result<bool, failure::Error> {
    info!("util.rs, logged_in()");
    match USERDATA.lock() {
        Ok(user_data) => {
            Ok(user_data.contains_key(client_id))
        }
        Err(_) => {
            Err(WebGuiError::UserDataMutexLockError.into())
        }
    }
}

pub fn check_login(login_id: &str, password: &str) -> Result<bool, failure::Error> {
    info!("util.rs, check_login()");
    let hash = get_hash_from_db(login_id)?;

    argon2::verify_encoded(&hash, password.as_bytes()).map_err(From::from)

/*
    use argon2::{self, Config};

    let password = b"password";
    let salt = b"randomsalt";
    let config = Config::default();
    let hash = argon2::hash_encoded(password, salt, &config).unwrap();
    let matches = argon2::verify_encoded(&hash, password).unwrap();
    assert!(matches);
*/
}

pub fn render<T: Serialize>(name: &str, context: &T) -> Result<String, failure::Error> {
    info!("util.rs, render()");
    TEMPLATE.render(name, context).map_err(From::from)
}

pub fn login_id(session_id: &str) -> Result<String, failure::Error> {
    info!("util.rs, login_id()");
    match USERDATA.lock() {
        Ok(user_data) => {
            match user_data.get(session_id).map(|data| data.login_id.clone()) {
                Some(login_id) => {
                    Ok(login_id)
                }
                None => {
                    Err(WebGuiError::LoginNotFound { session_id: session_id.to_string() }.into())
                }
            }
        }
        Err(_) => {
            Err(WebGuiError::UserDataMutexLockError.into())
        }
    }
}

pub fn login(session_id: &str, login_id: &str) -> Result<(), failure::Error> {
    info!("util.rs, login()");
    match USERDATA.lock() {
        Ok(mut user_data) => {
            match user_data.insert(session_id.to_string(),
            UserData { login_id: login_id.to_string() }).map(|data| data.login_id) {
                Some(login_id2) => {
                    Err(WebGuiError::AlreadyLoggedIn { session_id: session_id.to_string(), login_id: login_id.to_string(), login_id2 }.into())
                }
                None => {
                    Ok(())
                }
            }
        }
        Err(_) => {
            Err(WebGuiError::UserDataMutexLockError.into())
        }
    }
}

pub fn logout(session_id: &str) -> Result<String, failure::Error> {
    info!("util.rs, logout()");
    match USERDATA.lock() {
        Ok(mut user_data) => {
            match user_data.remove(session_id).map(|data| data.login_id) {
                Some(login_id) => {
                    Ok(login_id)
                }
                None => {
                    Err(WebGuiError::CouldNotLogout { session_id: session_id.to_string() }.into())
                }
            }
        }
        Err(_) => {
            Err(WebGuiError::UserDataMutexLockError.into())
        }
    }
}
