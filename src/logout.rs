use rouille::{Response};

use util;

pub fn handle(session_id: &str) -> Response {
    if util::logged_in(session_id) {
        util::logout(session_id);
        Response::html(util::render("logout", &()))
    } else {
        Response::html(util::render("login", &json!({"message": "Please log in first"})))
    }
}
