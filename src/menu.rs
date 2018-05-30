use rouille::{Response};

use util;

pub fn handle(session_id: &str) -> Response {
    if util::logged_in(session_id) {
        Response::html(util::render("menu", &json!({"login_id": util::login_id(session_id)})))
    } else {
        Response::html(util::render("login", &json!({"message": "Please log in first"})))
    }
}
