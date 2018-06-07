use rouille::{Response, Request};
use failure;

use util::{render, show_program};
use database::{check_login, login, login_id};
use programs::{ProgramType};

pub fn handle(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("login.rs, handle()");
    let data = post_input!(request, {
        login_id: String,
        password: String,
        program: u8,
    })?;

    Ok(if check_login(&data.login_id, &data.password)? {
        login(session_id, &data.login_id)?;
        Response::html(show_program(&data.login_id, Some(ProgramType::convert(data.program)?))?)
    } else {
        Response::html(render("login", &json!({"message": "Wrong user name or password", "login_error": "true"}))?)
    })
}
