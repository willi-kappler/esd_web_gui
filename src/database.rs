use std::sync::{Mutex, MutexGuard};
use std::process;
use std::convert::From;

use diesel::{self, SqliteConnection, Connection, QueryDsl, ExpressionMethods, RunQueryDsl};
use argon2;
use failure;

use configuration;
use error::{WebGuiError};
use program_types::{ProgramType};
use grain::{GrainImage};

table! {
    user_info {
        id -> Integer,
        login_id -> Text,
        session_id -> Text,
        logged_in -> Bool,
        full_name -> Text,
        email -> Text,
        passwd -> Text,
        is_active -> Bool,
        allowed_programs -> Text,
    }
}

table! {
    grain_images {
        id -> Integer,
        user_id -> Integer,
        path -> Text,
        sample_name -> Text,
        size -> Double,
        mode -> Integer,
        mineral -> Integer,
        ratio_232_238 -> Double,
        ratio_147_238 -> Double,
        orientation -> Integer,
        shape -> Integer,
        pyramids -> Integer,
        broken_tips -> Bool,
        zoned -> Bool,
        rim_width -> Double,
        ratio_rim_core -> Double,
    }
}

/*
    Database created with the following command:
    CREATE TABLE user_info(
        id INTEGER PRIMARY KEY ASC,
        is_active INTEGER,
        login_id TEXT,
        session_id TEXT,
        logged_in INTEGER,
        full_name TEXT,
        email TEXT,
        passwd TEXT,
        allowed_programs TEXT
    );

    CREATE TABLE grain_images(
        id INTEGER PRIMARY KEY ASC,
        user_id INTEGER,
        path TEXT,
        sample_name TEXT,
        size REAL,
        mode INTEGER,
        mineral INTEGER,
        ratio_232_238 REAL,
        ratio_147_238 REAL,
        orientation INTEGER,
        shape INTEGER,
        pyramids INTEGER,
        broken_tips INTEGER,
        zoned INTEGER,
        rim_width REAL,
        ratio_rim_core REAL
    );


    Test user added with the following command:
    insert into user_info (login_id, session_id, logged_in, full_name, email, passwd) values ("test_user", "", 0, "Test User", "test@home.com", "$argon2i$v=19$m=4096,t=3,p=1$hashvalue");
*/

lazy_static! {
    static ref DB_CONNECTION: Mutex<SqliteConnection> = {
        Mutex::new(SqliteConnection::establish(":memory:").unwrap())
    };
}

fn get_db_connection<'a>() -> Result<MutexGuard<'a, SqliteConnection>, failure::Error> {
    DB_CONNECTION.lock().map_err(|_| WebGuiError::DatabaseMutexLockError.into())
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
    let mut connection = get_db_connection()?;
    *connection = new_connection;
    Ok(())

    // SqliteConnection::establish(&database_url)

    // https://docs.rs/diesel/1.2.2/diesel/query_dsl/trait.QueryDsl.html
    // https://docs.rs/diesel/1.2.2/diesel/query_dsl/trait.RunQueryDsl.html
    // https://docs.rs/diesel/1.2.2/diesel/fn.insert_into.html
    // https://docs.rs/diesel/1.2.2/diesel/fn.update.html
    // https://docs.rs/diesel/1.2.2/diesel/sql_types/index.html
    //

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


    use schema::posts;

        let new_post = NewPost {
            title: title,
            body: body,
        };

        diesel::insert_into(posts::table)
            .values(&new_post)
            .execute(conn)
    .expect("Error saving new post")



    let num_deleted = diesel::delete(posts.filter(title.like(pattern)))
        .execute(&connection)
        .expect("Error deleting posts");


    let _ = diesel::update(posts.find(id))
            .set(published.eq(true))
            .execute(&connection)
            .expect(&format!("Unable to find post {}", id));


    let post: models::Post = posts
            .find(id)
            .first(&connection)
            .expect(&format!("Unable to find post {}", id));


    let seans_id = users.filter(name.eq("Sean")).select(id)
        .first(&connection);
    assert_eq!(Ok(1), seans_id);



    */

}

fn get_hash_from_db(form_login_id: &str) -> Result<Option<String>, failure::Error> {
    debug!("database.rs, get_hash_from_db()");
    use self::user_info::dsl::*;

    let connection = get_db_connection()?;
    let results : Vec<String> = user_info
        .filter(login_id.eq(form_login_id))
        .filter(is_active.eq(true))
        .select(passwd)
        .get_results(&*connection)?;
    let num_of_results = results.len();

    match num_of_results {
        0 => {
            Ok(None)
        }
        1 => {
            let hash = results[0].clone();
            Ok(Some(hash))
        }
        _ => {
            Err(WebGuiError::MultipleUsers.into())
        }
    }
}

pub fn logged_in(client_session_id: &str) -> Result<bool, failure::Error> {
    debug!("database.rs, logged_in()");
    use self::user_info::dsl::*;

    let connection = get_db_connection()?;
    let results : Vec<bool> = user_info
        .filter(session_id.eq(client_session_id))
        .select(logged_in)
        .get_results(&*connection)?;
    let num_of_results = results.len();

    match num_of_results {
        0 => {
            Ok(false)
        }
        1 => {
            Ok(results[0])
        }
        _ => {
            Err(WebGuiError::MultipleSessions.into())
        }
    }
}

pub fn check_login(login_id: &str, password: &str) -> Result<bool, failure::Error> {
    debug!("database.rs, check_login()");
    match get_hash_from_db(login_id)? {
        Some(hash) => {
            argon2::verify_encoded(&hash, password.as_bytes()).map_err(From::from)
        }
        None => {
            Ok(false)
        }
    }

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

pub fn login_id(client_session_id: &str) -> Result<(String, i32), failure::Error> {
    debug!("database.rs, login_id()");
    use self::user_info::dsl::*;

    let connection = get_db_connection()?;

    let results : Vec<(String, i32)> = user_info
        .filter(session_id.eq(client_session_id))
        .select((login_id, id))
        .get_results(&*connection)?;
    let num_of_results = results.len();

    match num_of_results {
        0 => {
            Err(WebGuiError::SessionNotFound.into())
        }
        1 => {
            Ok(results[0].clone())
        }
        _ => {
            Err(WebGuiError::MultipleSessions.into())
        }
    }
}

pub fn login(client_session_id: &str, form_login_id: &str) -> Result<(), failure::Error> {
    debug!("database.rs, login()");
    use self::user_info::dsl::*;

    let connection = get_db_connection()?;

    let results : Vec<i32> = user_info
        .filter(login_id.eq(form_login_id))
        .select(id)
        .get_results(&*connection)?;
    let num_of_results = results.len();

    match num_of_results {
        0 => {
            Err(WebGuiError::UserNotFound.into())
        }
        1 => {
            let row_id = results[0];
            let rows_affected : usize = diesel::update(user_info.filter(id.eq(row_id)))
                .set((session_id.eq(client_session_id), logged_in.eq(true)))
                .execute(&*connection)?;

            match rows_affected {
                1 => {
                    Ok(())
                }
                _ => {
                    Err(WebGuiError::UpdateDBError.into())
                }
            }
        }
        _ => {
            Err(WebGuiError::MultipleUsers.into())
        }
    }
}

pub fn logout(client_session_id: &str) -> Result<String, failure::Error> {
    debug!("database.rs, logout()");
    use self::user_info::dsl::*;

    let connection = get_db_connection()?;
    let results : Vec<i32> = user_info
        .filter(session_id.eq(client_session_id))
        .select(id)
        .get_results(&*connection)?;
    let num_of_results = results.len();

    match num_of_results {
        0 => {
            Err(WebGuiError::SessionNotFound.into())
        }
        1 => {
            let row_id = results[0];
            let rows_affected : usize = diesel::update(user_info.filter(id.eq(row_id)))
                .set((session_id.eq(""), logged_in.eq(false)))
                .execute(&*connection)?;

            match rows_affected {
                1 => {
                    let old_login_id : String = user_info
                        .filter(id.eq(row_id))
                        .select(login_id)
                        .get_result(&*connection)?;
                    Ok(old_login_id)
                }
                _ => {
                    Err(WebGuiError::UpdateDBError.into())
                }
            }
        }
        _ => {
            Err(WebGuiError::MultipleSessions.into())
        }
    }
}

pub fn log_out_everyone() -> Result<(), failure::Error>  {
    debug!("database.rs log_out_everyone()");
    use self::user_info::dsl::*;

    let connection = get_db_connection()?;
    let _rows_affected : usize = diesel::update(user_info)
        .set((session_id.eq(""), logged_in.eq(false)))
        .execute(&*connection)?;

    Ok(())
}

pub fn list_of_allowed_programs(user_db_id: i32) -> Result<Vec<ProgramType>, failure::Error> {
    debug!("database.rs, list_of_allowed_programs()");
    use self::user_info::dsl::*;

    let connection = get_db_connection()?;
    let results : Vec<String> = user_info
        .filter(id.eq(user_db_id))
        .select(allowed_programs)
        .get_results(&*connection)?;
    let num_of_results = results.len();

    match num_of_results {
        0 => {
            Err(WebGuiError::UserNotFound.into())
        }
        1 => {
            let programs = results[0].split(",")
                .map(|p| ProgramType::convert(p.parse::<u8>()?))
                .collect::<Result<Vec<_>, _>>()?;

            if programs.len() == 0 {
                Err(WebGuiError::NoProgramsForUser.into())
            } else {
                Ok(programs)
            }
        }
        _ => {
            Err(WebGuiError::MultipleUsers.into())
        }
    }
}

pub fn list_of_grain_images(user_db_id: i32) -> Result<Vec<GrainImage>, failure::Error> {
    debug!("database.rs, list_of_grain_images()");
    use self::grain_images::dsl::*;

    let connection = get_db_connection()?;
    let results : Vec<GrainImage> = grain_images
        .filter(user_id.eq(user_db_id))
        .order(sample_name.asc())
        .get_results(&*connection)?;

    Ok(results)
}

pub fn delete_grain_images(user_db_id: i32, image_ids: Vec<i32>) -> Result<(), failure::Error> {
    debug!("database.rs, delete_grain_images()");
    use self::grain_images::dsl::*;

    let connection = get_db_connection()?;
    // diesel::delete(table1::table.filter(table1::id.eq_any(vec![id1, id2, id3])).execute(conn);
    let _rows_affected : usize = diesel::delete(grain_images)
        .filter(id.eq_any(image_ids))
        .execute(&*connection)?;

    Ok(())
}

pub fn add_grain_image(user_db_id: i32, new_iamge: GrainImage) -> Result<(), failure::Error> {
    debug!("database.rs, delete_grain_images()");
    use self::grain_images::dsl::*;

    let connection = get_db_connection()?;
    let _rows_affected : usize = diesel::insert_into(grain_images).values((
        user_id.eq(user_db_id),
        path.eq(new_iamge.path),
        sample_name.eq(new_iamge.sample_name),
        size.eq(new_iamge.size),
        mode.eq(new_iamge.mode),
        mineral.eq(new_iamge.mineral),
        ratio_232_238.eq(new_iamge.ratio_232_238),
        ratio_147_238.eq(new_iamge.ratio_147_238),
        orientation.eq(new_iamge.orientation),
        shape.eq(new_iamge.shape),
        pyramids.eq(new_iamge.pyramids),
        broken_tips.eq(new_iamge.broken_tips),
        zoned.eq(new_iamge.zoned),
        rim_width.eq(new_iamge.rim_width),
        ratio_rim_core.eq(new_iamge.ratio_rim_core)
    )).execute(&*connection)?;

    Ok(())
}

pub fn list_of_grain_samples(user_db_id: i32) -> Result<Vec<String>, failure::Error> {
    debug!("database.rs, list_of_grain_images()");
    use self::grain_images::dsl::*;

    let connection = get_db_connection()?;
    let results : Vec<String> = grain_images
        .filter(user_id.eq(user_db_id))
        .select(sample_name)
        .distinct()
        .get_results(&*connection)?;

    Ok(results)
}
