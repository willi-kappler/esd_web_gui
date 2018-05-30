use rouille::{Response, Request};

use util;

pub fn handle(session_id: &str, request: &Request) -> Response {
    let data = try_or_400!(post_input!(request, {
        login_id: String,
        password: String,
        program: u8,
    }));

    if util::check_login(&data.login_id, &data.password) {
        util::login(session_id, &data.login_id);
        Response::html(util::render("main", &json!({"login_id": util::login_id(session_id)})))
    } else {
        Response::html(util::render("login", &json!({"message": "Wrong user name or password"})))
    }
}
