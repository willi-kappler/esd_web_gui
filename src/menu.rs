use rouille::{Response};
use failure;

use util::{logged_in, render, login_id};

pub fn handle(session_id: &str) -> Result<Response, failure::Error> {
    debug!("menu.rs, handle()");
    Ok(if logged_in(session_id)? {
        Response::html(render("menu", &json!({"login_id": login_id(session_id)?}))?)
    } else {
        Response::html(render("login", &json!({"message": "Please log in first"}))?)
    })
}
