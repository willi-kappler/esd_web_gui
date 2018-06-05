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
            db_name: "not_set".to_string(),
            db_user: "not_set".to_string(),
            db_password: "not_set".to_string(),
        })
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct Configuration {
    log_filename: String,
    db_name: String,
    db_user: String,
    db_password: String,
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

pub fn db_name() -> Result<String, failure::Error> {
    debug!("configuration.rs, db_name()");
    match CONFIGURATION.lock() {
        Ok(configuration) => {
            Ok(configuration.db_name.clone())
        }
        Err(_) => {
            Err(WebGuiError::ConfigurationMutexLockError.into())
        }
    }
}
