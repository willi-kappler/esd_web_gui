use std::sync::Mutex;
use std::process;
use std::convert::From;

use diesel::{SqliteConnection, Connection};
use argon2;
use failure;

use configuration;
use error::{WebGuiError};

#[derive(Queryable)]
struct UserInfo {
    id: i32,
    name: String,
    client_id: String,
    logged_in: bool,
    email: String,
    passwd: String,
}

table! {
    user_info (id) {
        id -> Integer,
        name -> Text,
        client_id -> Text,
        logged_in -> Bool,
        email -> Text,
        passwd -> Text,
    }
}

lazy_static! {
    static ref DB_CONNECTION: Mutex<SqliteConnection> = {
        Mutex::new(SqliteConnection::establish(":memory:").unwrap())
    };
}

pub fn connect_to_db() {
    debug!("database.rs, connect_to_db()");
    match connect_to_db_helper() {
        Ok(_) => {
            info!("Successfully connected to db");
        }
        Err(e) => {
            error!("Could not connect to db: {}", e);
            process::exit(1);
        }
    }

}

fn connect_to_db_helper() -> Result<(), failure::Error> {
    debug!("database.rs, connect_to_db_helper()");
    let new_connection = SqliteConnection::establish(&configuration::db_name()?)?;

    match DB_CONNECTION.lock() {
        Ok(mut connection) => {
            *connection = new_connection;
            Ok(())
        }
        Err(_) => {
            Err(WebGuiError::DatabaseMutexLockError.into())
        }
    }

    // SqliteConnection::establish(&database_url)

    /*


    fn main() {
        use self::schema::posts::dsl::*;

        let connection = establish_connection();
        let results = posts
            .filter(published.eq(true))
            .limit(5)
            .load::<Post>(&connection)
            .expect("Error loading posts");

        println!("Displaying {} posts", results.len());
        for post in results {
            println!("{}", post.title);
            println!("----------\n");
            println!("{}", post.body);
        }
    }


    */

}

fn get_hash_from_db(login_id: &str) -> Result<String, failure::Error> {
    debug!("database.rs, get_hash_from_db()");
    match DB_CONNECTION.lock() {
        Ok(connection) => {
            // TODO!!!
            Ok("$argon2i$v=19$m=4096,t=3,p=1$cm9oYmF1Y2hhYzlUdW8wY2k2UmF1bmd1aGFpZzVzb2hjb29Ob2hjaXdlcmVlczRiYWtlZXRoM0NvaGJpZUxhaA$KAta8FGbVMSv/OsA/PGL0FXrNfjJ4Gv6SUkaiZKYbHA".to_string())
        }
        Err(_) => {
            Err(WebGuiError::DatabaseMutexLockError.into())
        }
    }
}

pub fn logged_in(client_id: &str) -> Result<bool, failure::Error> {
    debug!("database.rs, logged_in()");
    match DB_CONNECTION.lock() {
        Ok(connection) => {
            // TODO
            Ok(false)
        }
        Err(_) => {
            Err(WebGuiError::DatabaseMutexLockError.into())
        }
    }
}

pub fn check_login(login_id: &str, password: &str) -> Result<bool, failure::Error> {
    debug!("database.rs, check_login()");
    let hash = get_hash_from_db(login_id)?;

    argon2::verify_encoded(&hash, password.as_bytes()).map_err(From::from)

/*
    use argon2::{self, Config};

    let password = b"password";
    let salt = b"randomsalt";
    let config = Config::default();
    let hash = argon2::hash_encoded(password, salt, &config).unwrap();
    let matches = argon2::verify_encoded(&hash, password).unwrap();
    assert!(matches);
*/
}

pub fn login_id(session_id: &str) -> Result<String, failure::Error> {
    debug!("database.rs, login_id()");
    match DB_CONNECTION.lock() {
        Ok(connection) => {
            // TODO
            Ok("no_user".to_string())
        }
        Err(_) => {
            Err(WebGuiError::DatabaseMutexLockError.into())
        }
    }
}

pub fn login(session_id: &str, login_id: &str) -> Result<(), failure::Error> {
    debug!("database.rs, login()");
    match DB_CONNECTION.lock() {
        Ok(connection) => {
            // TODO
            Ok(())
        }
        Err(_) => {
            Err(WebGuiError::DatabaseMutexLockError.into())
        }
    }
}

pub fn logout(session_id: &str) -> Result<String, failure::Error> {
    debug!("database.rs, logout()");
    match DB_CONNECTION.lock() {
        Ok(connection) => {
            // TODO
            Ok("no_user".to_string())
        }
        Err(_) => {
            Err(WebGuiError::DatabaseMutexLockError.into())
        }
    }
}
