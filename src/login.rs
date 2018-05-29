use rouille::{Response, Request};

use util;

pub fn handle(client_id: &str, request: &Request) -> Response {
    let data = try_or_400!(post_input!(request, {
        login_id: String,
        password: String,
        program: u8,
    }));

    if util::check_login(&data.login_id, &data.password) {
        Response::html(util::TEMPLATE.render("main", &json!({"login_id": data.login_id})).unwrap())
    } else {
        Response::html(util::TEMPLATE.render("login", &json!({"message": "Wrong user name or password"})).unwrap())
    }
}
