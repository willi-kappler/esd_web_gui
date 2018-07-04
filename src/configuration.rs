use std::sync::Mutex;
use std::process;
use std::env;
use std::fs;

use toml;
use failure;

use error::{WebGuiError};

lazy_static! {
    static ref CONFIGURATION : Mutex<Configuration> = {
        Mutex::new(Configuration {
            log_filename: "webgui.log".to_string(),
            user_db: "not_set".to_string(),
            grain_db: "not_set".to_string(),
        })
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct Configuration {
    log_filename: String,
    user_db: String,
    grain_db: String,
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

        match CONFIGURATION.lock() {
            Ok(mut configuration) => {
                *configuration = new_configuration;
                Ok(())
            }
            Err(_) => {
                Err(WebGuiError::ConfigurationMutexLockError.into())
            }
        }
    } else {
        println!("Usage: {} config_filename", input[0]);
        Err(WebGuiError::InvalidCommandLineArguments.into())
    }
}

pub fn log_filename() -> String {
    debug!("configuration.rs, log_filename()");
    match CONFIGURATION.lock() {
        Ok(configuration) => {
            configuration.log_filename.clone()
        }
        Err(e) => {
            println!("Could not lock CONFIGURATION: {}", e);
            process::exit(1)
        }
    }
}

pub fn user_db() -> Result<String, failure::Error> {
    debug!("configuration.rs, user_db()");
    match CONFIGURATION.lock() {
        Ok(configuration) => {
            Ok(configuration.user_db.clone())
        }
        Err(_) => {
            Err(WebGuiError::ConfigurationMutexLockError.into())
        }
    }
}

pub fn grain_db() -> Result<String, failure::Error> {
    debug!("configuration.rs, grain_db()");
    match CONFIGURATION.lock() {
        Ok(configuration) => {
            Ok(configuration.grain_db.clone())
        }
        Err(_) => {
            Err(WebGuiError::ConfigurationMutexLockError.into())
        }
    }
}
