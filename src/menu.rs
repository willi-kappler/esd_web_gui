use rouille::{Response};
use failure;

use util::{render, show_program};
use database::{logged_in, login_id};

pub fn handle(session_id: &str) -> Result<Response, failure::Error> {
    debug!("menu.rs, handle()");
    Ok(if logged_in(session_id)? {
        Response::html(show_program(&login_id(session_id)?, None)?)
    } else {
        Response::html(render("login", &json!({"message": "Please log in first"}))?)
    })
}
