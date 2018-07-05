use std::sync::{Mutex, MutexGuard};
use std::fs::File;
use std::io::{Read, BufReader};

use serde::{Serialize};
use handlebars::{Handlebars};
use failure;
use rouille::{Response};
use argon2;
use toml;

use program_types::{ProgramType};
use error::{WebGuiError};
use configuration;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct User {
    id: u16,
    is_active: bool,
    login_id: String,
    session_id: String,
    logged_in: bool,
    full_name: String,
    email: String,
    passwd: String,
    allowed_programs: Vec<ProgramType>,
}

lazy_static! {
    static ref TEMPLATE : Handlebars = {
        let mut hb = Handlebars::new();
        hb.register_template_file("login", "html/login.hbs").unwrap();
        hb.register_template_file("header", "html/header.hbs").unwrap();
        hb.register_template_file("footer", "html/footer.hbs").unwrap();
        hb.register_template_file("pecube", "html/pecube.hbs").unwrap();
        hb.register_template_file("grain", "html/grain.hbs").unwrap();
        hb.register_template_file("grain_load_images", "html/grain_load_images.hbs").unwrap();
        hb.register_template_file("grain_outline_images", "html/grain_outline_images.hbs").unwrap();
        hb.register_template_file("landlab", "html/landlab.hbs").unwrap();
        hb.register_template_file("icecascade", "html/icecascade.hbs").unwrap();
        hb.register_template_file("coupled", "html/coupled.hbs").unwrap();
        hb
    };

    static ref USER_DB : Mutex<Vec<User>> = {
        Mutex::new(Vec::new())
    };
}

fn get_db_lock<'a>() -> Result<MutexGuard<'a, Vec<User>>, failure::Error> {
    USER_DB.lock().map_err(|_| WebGuiError::UserDBMutexLockError.into())
}

pub fn load_db() -> Result<(), failure::Error> {
    debug!("utils.rs, load_db()");
    let mut user_db = get_db_lock()?;

    let mut data = String::new();
    let f = File::open(configuration::user_db()?)?;
    let mut f = BufReader::new(f);
    f.read_to_string(&mut data)?;

    *user_db = toml::from_str(&data)?;
    Ok(())
}

fn get_hash_from_db(login_id: &str) -> Result<Option<String>, failure::Error> {
    debug!("utils.rs, get_hash_from_db()");
    let user_db = get_db_lock()?;

    let passwd_hash = user_db.iter()
        .filter(|user| user.login_id == login_id)
        .map(|user| user.passwd.clone()).collect::<Vec<_>>();

    match passwd_hash.len() {
        0 => Ok(None),
        1 => Ok(Some(passwd_hash[0].clone())),
        _ => Err(WebGuiError::MultipleUsers.into()),
    }
}

pub fn check_login(login_id: &str, password: &str) -> Result<bool, failure::Error> {
    debug!("utils.rs, check_login()");
    match get_hash_from_db(login_id)? {
        Some(hash) => {
            argon2::verify_encoded(&hash, password.as_bytes()).map_err(From::from)
        }
        None => {
            Ok(false)
        }
    }
}

pub fn login(session_id: &str, login_id: &str) -> Result<(), failure::Error> {
    debug!("utils.rs, login()");
    let mut user_db = get_db_lock()?;

    let mut users_found = 0;
    let mut index = 0;

    for i in 0..user_db.len() {
        if user_db[i].session_id == session_id && user_db[i].login_id == login_id {
            users_found += 1;
            index = i;
        }
    }

    match users_found {
        0 => Err(WebGuiError::UserNotFound.into()),
        1 => {
            user_db[index].logged_in = true;
            Ok(())
        }
        _ => Err(WebGuiError::MultipleUsers.into()),
    }
}

pub fn logout(session_id: &str) -> Result<(), failure::Error> {
    debug!("utils.rs, logout()");
    let mut user_db = get_db_lock()?;

    let mut users_found = 0;
    let mut index = 0;

    for i in 0..user_db.len() {
        if user_db[i].session_id == session_id {
            users_found += 1;
            index = i;
        }
    }

    match users_found {
        0 => Err(WebGuiError::UserNotFound.into()),
        1 => {
            user_db[index].logged_in = false;
            Ok(())
        }
        _ => Err(WebGuiError::MultipleUsers.into()),
    }
}

pub fn logged_in(session_id: &str) -> Result<bool, failure::Error> {
    debug!("utils.rs, logged_in()");
    let user_db = get_db_lock()?;

    let users_logged_in = user_db.iter()
        .filter(|user| user.session_id == session_id)
        .map(|user| user.logged_in).collect::<Vec<_>>();

    match users_logged_in.len() {
        0 => Err(WebGuiError::UserNotFound.into()),
        1 => Ok(users_logged_in[0]),
        _ => Err(WebGuiError::MultipleUsers.into()),
    }
}

pub fn login_id(session_id: &str) -> Result<(String, u16), failure::Error> {
    debug!("utils.rs, login_id()");
    let user_db = get_db_lock()?;

    let user_ids = user_db.iter()
        .filter(|user| user.session_id == session_id)
        .map(|user| (user.login_id.clone(), user.id)).collect::<Vec<_>>();

    match user_ids.len() {
        0 => Err(WebGuiError::UserNotFound.into()),
        1 => Ok(user_ids[0].clone()),
        _ => Err(WebGuiError::MultipleUsers.into()),
    }
}

pub fn list_of_allowed_programs(user_id: u16) -> Result<Vec<ProgramType>, failure::Error> {
    debug!("utils.rs, login_id()");
    let user_db = get_db_lock()?;

    let allowed_programs = user_db.iter()
        .filter(|user| user.id == user_id)
        .map(|user| user.allowed_programs.clone()).collect::<Vec<_>>();

    match allowed_programs.len() {
        0 => Err(WebGuiError::UserNotFound.into()),
        1 => Ok(allowed_programs[0].clone()),
        _ => Err(WebGuiError::MultipleUsers.into()),
    }
}

pub fn render<T: Serialize>(name: &str, context: &T) -> Result<String, failure::Error> {
    debug!("util.rs, render()");
    TEMPLATE.render(name, context).map_err(|e| e.into())
}

pub fn get_template_name<'a>(program: &ProgramType) -> &'a str {
    debug!("util.rs, get_template_name()");
    use self::ProgramType::*;

    match program {
        PecubeESD => "pecube",
        Grain3DHe => "grain",
        LandLabESD => "landlab",
        IceCascade => "icecascade",
        CoupledLandscapeThermalSimulator => "coupled",
    }
}

pub fn get_menu_name<'a>(program: &ProgramType) -> &'a str {
    debug!("util.rs, get_menu_name()");
    use self::ProgramType::*;

    match program {
        PecubeESD => "PecubeESD",
        Grain3DHe => "3D-He",
        LandLabESD => "LandLab",
        IceCascade => "IceCascade",
        CoupledLandscapeThermalSimulator => "CoupledLandscape",
    }
}

pub fn build_program_menu(allowed_programs: &Vec<ProgramType>) -> Vec<(&str, &str)> {
    allowed_programs.iter().map(|p| (get_template_name(p), get_menu_name(p))).collect::<Vec<_>>()
}

pub fn show_program(session_id: &str, program: &ProgramType) -> Result<Response, failure::Error> {
    debug!("util.rs, show_program()");

    if logged_in(session_id)? {
        let (user_name, db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(db_id)?;

        if allowed_programs.contains(&program) {
            let user_menu = json!({"login_id": user_name, "programs": build_program_menu(&allowed_programs)});
            debug!("user_menu: {}", user_menu);
            Ok(Response::html(render(get_template_name(program), &user_menu)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn replace_characters(input: &str) -> String {
    input.chars().filter(|c|
        c.is_ascii_alphanumeric() || *c == '_' || *c == '.' || *c == '-'
    ).collect()
}
