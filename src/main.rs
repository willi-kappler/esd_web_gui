#[macro_use] extern crate rouille;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate diesel;

extern crate handlebars;
extern crate serde;
extern crate argon2;
extern crate log4rs;
extern crate toml;

mod menu;
mod login;
mod logout;
mod database;
mod configuration;
mod error;
mod util;
mod programs;

use std::fs::File;

use rouille::{Request, Response};

fn main() {
    /*
    let password = b"test1";
    let salt = b"rohbauchac9Tuo0ci6Raunguhaig5sohcooNohciwerees4bakeeth3CohbieLah";
    let config = argon2::Config::default();
    let hash = argon2::hash_encoded(password, salt, &config).unwrap();
    println!("hash: {}", hash);
    return;
    */

    configuration::load_configuration();

    let file_logger = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d} {l} - {m}{n}")))
        .build(configuration::log_filename()).unwrap();

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("file_logger", Box::new(file_logger)))
        .build(log4rs::config::Root::builder().appender("file_logger").build(log::LevelFilter::Debug))
        .unwrap();

    let _log_handle = log4rs::init_config(config).unwrap();

    database::connect_to_db();

    let addr = "0.0.0.0:3030";
    println!("Now listening on {}", addr);

    rouille::start_server(addr, move |request| {
        rouille::session::session(request, "ESD", 3600, |session| {
            let session_id = session.id();

            match handle_request(request, session_id) {
                Ok(response) => {
                    response
                }
                Err(e) => {
                    error!("main.rs, handle_request: An error occured: {}", e);
                    Response::empty_404()
                }
            }

        })
    });
}

fn handle_request(request: &Request, session_id: &str) -> Result<Response, failure::Error> {
    debug!("main.rs, handle_request()");

    Ok(router!(request,
        (GET) ["/"] => {
            menu::handle(session_id)?
        },
        (GET) ["/logout"] => {
            logout::handle(session_id)?
        },
        (POST) ["/login"] => {
            login::handle(session_id, request)?
        },

        // Static files:
        (GET) ["/images/uni_esd_logo.jpg"] => {
            let file = File::open("images/uni_esd_logo.jpg")?;
            Response::from_file("image/jpeg", file)
        },
        (GET) ["/css/login.css"] => {
            let file = File::open("css/login.css")?;
            Response::from_file("text/css", file)
        },
        _ => Response::empty_404()
    ))
}
