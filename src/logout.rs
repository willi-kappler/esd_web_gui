use rouille::{Response};

use util;

pub fn handle(client_id: &str) -> Response {
    if util::logged_in(client_id) {
        Response::html(util::TEMPLATE.render("logout", &()).unwrap())
    } else {
        Response::html(util::TEMPLATE.render("login", &json!({"message": "Please log in first"})).unwrap())
    }
}
