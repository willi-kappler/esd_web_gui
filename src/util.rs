use std::collections::HashMap;
use std::sync::Mutex;
use std::convert::From;
use std::process;
use std::fs;
use std::env;

use serde::{Serialize};
use handlebars::{Handlebars};
use argon2;
use toml;

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

    static ref CONFIGURATION: Mutex<Configuration> = {
        Mutex::new(Configuration {
            log_filename: "webgui.log".to_string(),
            db_name: "not_set".to_string(),
            db_user: "not_set".to_string(),
            db_password: "not_set".to_string(),
        })
    };
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct UserData {
    login_id: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct Configuration {
    log_filename: String,
    db_name: String,
    db_user: String,
    db_password: String,
}

pub fn load_configuration() {
    let input: Vec<String> = env::args().collect();

    debug!("util.rs, load_configuration()");

    match load_configuration_helper(input) {
        Ok(_) => {
            println!("Configuration loaded successfully");
        }
        Err(e) => {
            println!("Error loading configuration file: {}", e);
            process::exit(1);
        }
    }
}

fn load_configuration_helper(input: Vec<String>) -> Result<(), failure::Error> {
    debug!("util.rs, load_configuration_helper()");
    if input.len() == 2 {
        let filename = &input[1];
        println!("Try to open file '{}'", filename);
        let content = fs::read_to_string(filename)?;
        let config : Configuration = toml::from_str(&content)?;

        match CONFIGURATION.lock() {
            Ok(mut configuration) => {
                *configuration = config;
                Ok(())
            }
            Err(_) => {
                Err(WebGuiError::ConfigurationMutexLockError.into())
            }
        }
    } else {
        println!("Usage: {} config_filename", input[0]);
        Err(WebGuiError::InvalidCommandLineArguments.into())
    }
}

pub fn log_filename() -> String {
    debug!("util.rs, log_filename()");
    match CONFIGURATION.lock() {
        Ok(configuration) => {
            configuration.log_filename.clone()
        }
        Err(e) => {
            println!("Could not lock CONFIGURATION: {}", e);
            process::exit(1)
        }
    }
}

fn get_hash_from_db(login_id: &str) -> Result<String, failure::Error> {
    debug!("util.rs, get_hash_from_db()");
    // TODO!!!
    Ok("$argon2i$v=19$m=4096,t=3,p=1$cm9oYmF1Y2hhYzlUdW8wY2k2UmF1bmd1aGFpZzVzb2hjb29Ob2hjaXdlcmVlczRiYWtlZXRoM0NvaGJpZUxhaA$KAta8FGbVMSv/OsA/PGL0FXrNfjJ4Gv6SUkaiZKYbHA".to_string())
}

pub fn logged_in(client_id: &str) -> Result<bool, failure::Error> {
    debug!("util.rs, logged_in()");
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
    debug!("util.rs, check_login()");
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
    debug!("util.rs, render()");
    TEMPLATE.render(name, context).map_err(From::from)
}

pub fn login_id(session_id: &str) -> Result<String, failure::Error> {
    debug!("util.rs, login_id()");
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
    debug!("util.rs, login()");
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
    debug!("util.rs, logout()");
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
