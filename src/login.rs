use rouille::{Response, Request};
use failure;

use util;
use program_types::{ProgramType};

pub fn handle(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("login.rs, handle()");
    let data = post_input!(request, {
        login_id: String,
        password: String,
        program: u8,
    })?;

    Ok(if util::check_login(&data.login_id, &data.password)? {
        util::login(session_id, &data.login_id)?;
        Response::redirect_303(util::get_template_name(&ProgramType::convert(data.program)?))
    } else {
        Response::html(util::render("login", &json!({"message": "Wrong user name or password", "login_error": "true"}))?)
    })
}
