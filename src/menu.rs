use rouille::{Response};
use failure;

use util;

pub fn handle(session_id: &str) -> Result<Response, failure::Error> {
    debug!("menu.rs, handle()");
    Ok(if util::logged_in(session_id)? {
        Response::redirect_303("pecube")
    } else {
        Response::html(util::render("login", &json!({"message": "Please log in first"}))?)
    })
}
