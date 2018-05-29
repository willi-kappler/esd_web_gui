#[macro_use] extern crate rouille;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;
extern crate handlebars;

mod login;
mod logout;
mod util;

use rouille::{Response};

fn main() {
    let addr = "0.0.0.0:3030";
    println!("Now listening on {}", addr);

    rouille::start_server(addr, move |request| {
        rouille::session::session(request, "ESD", 3600, |session| {
            router!(request,
                (GET) (/) => {
                    if util::logged_in(session.id()) {
                        Response::html(util::TEMPLATE.render("main", &()).unwrap())
                    } else {
                        Response::html(util::TEMPLATE.render("login", &json!({"message": "Please log in first"})).unwrap())
                    }
                },
                (GET) (/logout) => {
                    logout::handle(session.id())
                },
                (POST) (/login) => {
                    login::handle(session.id(), request)
                },
                _ => Response::empty_404()
            )
        })
    });
}
