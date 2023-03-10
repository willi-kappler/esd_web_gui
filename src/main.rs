#[macro_use] extern crate rouille;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;

extern crate handlebars;
extern crate serde;
extern crate argon2;
extern crate log4rs;
extern crate toml;
extern crate image;
extern crate itertools;

// Request handler:
mod menu;
mod login;
mod logout;
mod programs;

// Helper / utils:
mod configuration;
mod error;
mod util;
mod program_types;

use std::fs::File;

use rouille::{Request, Response};

use programs::{pecube, grain, landlab, icecascade, coupled};

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

    util::load_db().unwrap();
    grain::load_db().unwrap();


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
                    Response::html("An error occured. Please contact the administrator")
                }
            }

        })
    });
}

fn handle_request(request: &Request, session_id: &str) -> Result<Response, failure::Error> {
    debug!("main.rs, handle_request()");

    Ok(router!(request,
        (GET) ["/web_gui/"] => {
            menu::handle(session_id)?
        },
        (POST) ["/web_gui/"] => {
            login::handle(session_id, request)?
        },
        (GET) ["/web_gui/logout"] => {
            logout::handle(session_id)?
        },

        // Pecube:
        (GET) ["/web_gui/pecube"] => {
            pecube::about_get(session_id)?
        },

        // 3D He (FT Grain Correction):
        (GET) ["/web_gui/grain"] => {
            grain::about_get(session_id)?
        },
        (GET) ["/web_gui/grain/load_images"] => {
            grain::load_images_get(session_id)?
        },
        (POST) ["/web_gui/grain/load_images"] => {
            grain::load_images_post(session_id, request)?
        },
        (POST) ["/web_gui/grain/remove_images"] => {
            grain::remove_images_post(session_id, request)?
        },
        (GET) ["/web_gui/grain/outline_images"] => {
            grain::outline_images_get(session_id)?
        },
        (POST) ["/web_gui/grain/outline_images"] => {
            grain::outline_images_post(session_id, request)?
        },
        (POST) ["/web_gui/grain/store_outlines"] => {
            grain::store_outline_post(session_id, request)?
        },
        (GET) ["/web_gui/grain/calculate"] => {
            grain::calculate_get(session_id)?
        },
        (POST) ["/web_gui/grain/calculate"] => {
            grain::calculate_post(session_id, request)?
        },

        (GET) ["/web_gui/grain/user_data/{username}/{samplename}/{imagename}", username: String, samplename: String, imagename: String] => {
            grain::sample_image_get(session_id, username, samplename, imagename)?
        },
        (GET) ["/web_gui/js/grain_outline.js"] => {
            let file = File::open("js/grain_outline.js")?;
            Response::from_file("text/javascript", file)
        },
        (GET) ["/web_gui/js/grain_refresh.js"] => {
            let file = File::open("js/grain_refresh.js")?;
            Response::from_file("text/javascript", file)
        },


        // Landlab:
        (GET) ["/web_gui/landlab"] => {
            landlab::about_get(session_id)?
        },

        // IceCascade:
        (GET) ["/web_gui/icecascade"] => {
            icecascade::about_get(session_id)?
        },

        // Coupled:
        (GET) ["/web_gui/coupled"] => {
            coupled::about_get(session_id)?
        },

        // Static files:
        (GET) ["/web_gui/images/uni_esd_logo.jpg"] => {
            let file = File::open("images/uni_esd_logo.jpg")?;
            Response::from_file("image/jpeg", file)
        },
        (GET) ["/web_gui/images/grain20_photo.jpg"] => {
            let file = File::open("images/grain20_photo.jpg")?;
            Response::from_file("image/jpeg", file)
        },
        (GET) ["/web_gui/images/plus.png"] => {
            let file = File::open("images/plus.png")?;
            Response::from_file("image/png", file)
        },
        (GET) ["/web_gui/images/minus.png"] => {
            let file = File::open("images/minus.png")?;
            Response::from_file("image/png", file)
        },
        (GET) ["/web_gui/css/login.css"] => {
            let file = File::open("css/login.css")?;
            Response::from_file("text/css", file)
        },
        (GET) ["/web_gui/css/menu.css"] => {
            let file = File::open("css/menu.css")?;
            Response::from_file("text/css", file)
        },
        _ => {
            debug!("Page not found: {}", request.raw_url());
            Response::html("Web-GUI: Page not found")
        }
    ))
}
