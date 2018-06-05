use rouille::{Response};
use failure;

use util::{render};
use database::{logged_in, logout};

pub fn handle(session_id: &str) -> Result<Response, failure::Error> {
    debug!("logout.rs, handle()");
    Ok(if logged_in(session_id)? {
        logout(session_id)?;
        Response::html(render("logout", &())?)
    } else {
        Response::html(render("login", &json!({"message": "Please log in first"}))?)
    })
}
