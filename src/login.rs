use rouille::{Response, Request};
use failure;

use util::{check_login, login, render, login_id};

pub fn handle(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    info!("login.rs, handle()");
    let data = post_input!(request, {
        login_id: String,
        password: String,
        program: u8,
    })?;

    Ok(if check_login(&data.login_id, &data.password)? {
        login(session_id, &data.login_id)?;
        Response::html(render("menu", &json!({"login_id": login_id(session_id)?, "program": &data.program}))?)
    } else {
        Response::html(render("login", &json!({"message": "Wrong user name or password"}))?)
    })
}
