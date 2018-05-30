use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Serialize};
use handlebars::{Handlebars};

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

pub fn logged_in(client_id: &str) -> bool {
    let user_data = USERDATA.lock().unwrap();
    user_data.contains_key(client_id)
}

pub fn check_login(login_id: &str, password: &str) -> bool {
    // TODO

/*
    use argon2::{self, Config};

    let password = b"password";
    let salt = b"randomsalt";
    let config = Config::default();
    let hash = argon2::hash_encoded(password, salt, &config).unwrap();
    let matches = argon2::verify_encoded(&hash, password).unwrap();
    assert!(matches);
*/

    true
}

pub fn render<T: Serialize>(name: &str, context: &T) -> String {
    match TEMPLATE.render(name, context) {
        Ok(page) => { page }
        Err(e) => { format!("util:render: TEMPLATE error: {:?}", e) }
    }
}

pub fn login_id(session_id: &str) -> Option<String> {
    let user_data = USERDATA.lock().unwrap();
    user_data.get(session_id).map(|data| data.login_id.clone())
}

pub fn login(session_id: &str, login_id: &str) -> Option<String> {
    let mut user_data = USERDATA.lock().unwrap();
    user_data.insert(session_id.to_string(), UserData { login_id: login_id.to_string() }).map(|data| data.login_id)
}

pub fn logout(session_id: &str) -> Option<String> {
    let mut user_data = USERDATA.lock().unwrap();
    user_data.remove(session_id).map(|data| data.login_id)
}
