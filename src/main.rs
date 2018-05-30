#[macro_use] extern crate rouille;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;
extern crate handlebars;
extern crate serde;

mod menu;
mod login;
mod logout;
mod util;

use rouille::{Response};

fn main() {
    let addr = "0.0.0.0:3030";
    println!("Now listening on {}", addr);

    rouille::start_server(addr, move |request| {
        rouille::session::session(request, "ESD", 3600, |session| {
            let session_id = session.id();

            router!(request,
                (GET) (/) => {
                    menu::handle(session_id)
                },
                (GET) (/logout) => {
                    logout::handle(session_id)
                },
                (POST) (/login) => {
                    login::handle(session_id, request)
                },
                _ => Response::empty_404()
            )
        })
    });
}
