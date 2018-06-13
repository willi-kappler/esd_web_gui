use rouille::{Response, Request};
use failure;

use util::{render, show_program, build_user_menu, get_template_name};
use program_types::{ProgramType};
use database::{login_id, logged_in, list_of_allowed_programs};

pub fn about_get(session_id: &str) -> Result<Response, failure::Error> {
    show_program(session_id, &ProgramType::Grain3DHe)
}

pub fn load_images_get(session_id: &str) -> Result<Response, failure::Error> {
    if logged_in(session_id)? {
        let user_login_id = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(&user_login_id)?;
        let context = build_user_menu(&user_login_id, &allowed_programs);

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            Ok(Response::html(render("grain_load_images", &context)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn load_images_post(session_id: &str, _request: &Request) -> Result<Response, failure::Error> {
    // TODO
    let _user_login_id = login_id(session_id)?;
    Ok(Response::redirect_303("/grain/load_images"))
}
