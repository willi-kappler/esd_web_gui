use rouille::{Response};
use failure;

use util::{render};
use database::{logged_in};

pub fn handle(session_id: &str) -> Result<Response, failure::Error> {
    debug!("menu.rs, handle()");
    Ok(if logged_in(session_id)? {
        Response::redirect_303("menu_pecube")
    } else {
        Response::html(render("login", &json!({"message": "Please log in first"}))?)
    })
}
