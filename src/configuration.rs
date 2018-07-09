use std::sync::{Mutex, MutexGuard};
use std::process;
use std::env;
use std::fs;
use std::{thread, time};

use toml;
use failure;

use error::{WebGuiError};

lazy_static! {
    static ref CONFIGURATION : Mutex<Configuration> = {
        Mutex::new(Configuration {
            log_filename: "webgui.log".to_string(),
            user_db: "not_set".to_string(),
            grain_db: "not_set".to_string(),
            matlab_exec: "not_set".to_string(),
            matlab_folder: "not_set".to_string(),
        })
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct Configuration {
    log_filename: String,
    user_db: String,
    grain_db: String,
    matlab_exec: String,
    matlab_folder: String,
}

fn get_db_lock<'a>() -> MutexGuard<'a, Configuration> {
    loop {
        let lock = CONFIGURATION.try_lock();
        if let Ok(mutex) = lock {
            return mutex
        } else {
            debug!("configuration.rs, get_db_lock() -> thread_sleep");
            // Sleep and try again to aquire the lock
            let duration = time::Duration::from_millis(100);
            thread::sleep(duration);
        }
    }
}



pub fn load_configuration() {
    debug!("configuration.rs, load_configuration()");
    let input: Vec<String> = env::args().collect();

    match load_configuration_helper(input) {
        Ok(_) => {
            println!("Configuration loaded successfully");
        }
        Err(e) => {
            println!("Error loading configuration file: {}", e);
            process::exit(1);
        }
    }
}

fn load_configuration_helper(input: Vec<String>) -> Result<(), failure::Error> {
    debug!("configuration.rs, load_configuration_helper()");
    if input.len() == 2 {
        let filename = &input[1];
        println!("Try to open file '{}'", filename);
        let content = fs::read_to_string(filename)?;
        let new_configuration : Configuration = toml::from_str(&content)?;

        let mut configuration = get_db_lock();
        *configuration = new_configuration;
        Ok(())
    } else {
        println!("Usage: {} config_filename", input[0]);
        Err(WebGuiError::InvalidCommandLineArguments.into())
    }
}

pub fn log_filename() -> String {
    debug!("configuration.rs, log_filename()");
    let configuration = get_db_lock();
    configuration.log_filename.clone()
}

pub fn user_db() -> String {
    debug!("configuration.rs, user_db()");
    let configuration = get_db_lock();
    configuration.user_db.clone()
}

pub fn grain_db() -> String {
    debug!("configuration.rs, grain_db()");
    let configuration = get_db_lock();
    configuration.grain_db.clone()
}

pub fn matlab_exec() -> String {
    debug!("configuration.rs, grain_db()");
    let configuration = get_db_lock();
    configuration.matlab_exec.clone()
}

pub fn matlab_folder() -> String {
    debug!("configuration.rs, grain_db()");
    let configuration = get_db_lock();
    configuration.matlab_folder.clone()
}
