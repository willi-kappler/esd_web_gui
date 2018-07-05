use std::sync::Mutex;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, BufReader, Write, Read};

use rouille::{Response, Request, input};
use failure;
use image::{self, GenericImage};
use toml;

use util;
use program_types::{ProgramType};
use error::{WebGuiError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Axis {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct GrainImage {
    id: u32,
    user_id: u16,
    path: String,
    sample_name: String,
    size: f64,
    mode: i32,
    mineral: i32,
    ratio_232_238: f64,
    ratio_147_238: f64,
    orientation: i32,
    shape: i32,
    pyramids: i32,
    broken_tips: bool,
    zoned: bool,
    rim_width: f64,
    ratio_rim_core: f64,
    coordinates: Vec<(u32, u32)>,
    axis: Axis,
}

lazy_static! {
    static ref GRAIN_DB : Mutex<Vec<GrainImage>> = {
        Mutex::new(Vec::new())
    };
}

pub fn load_db(filename: &str) -> Result<(), failure::Error> {
    debug!("utils.rs, load_db()");
    match GRAIN_DB.lock() {
        Ok(mut grain_db) => {
            let mut data = String::new();
            let f = File::open(filename)?;
            let mut f = BufReader::new(f);
            f.read_to_string(&mut data)?;

            *grain_db = toml::from_str(&data)?;
            Ok(())
        }
        Err(_) => {
            Err(WebGuiError::GrainDBMutexLockError.into())
        }
    }
}

fn list_of_grain_images(user_id: u16) -> Result<Vec<GrainImage>, failure::Error> {
    debug!("grain.rs, list_of_grain_images()");
    match GRAIN_DB.lock() {
        Ok(grain_db) => {
            Ok(grain_db.iter().filter(|grain| grain.user_id == user_id).map(|grain| grain.clone()).collect())
        }
        Err(_) => {
            Err(WebGuiError::GrainDBMutexLockError.into())
        }
    }
}

fn list_of_grain_samples(user_id: u16) -> Result<Vec<String>, failure::Error> {
    debug!("grain.rs, list_of_grain_samples()");

    Ok(Vec::new())
}

fn add_grain_image(user_id: u16, new_iamge: GrainImage) -> Result<(), failure::Error> {
    debug!("grain.rs, add_grain_images()");

    Ok(())
}

fn delete_grain_images(user_id: u16, image_ids: Vec<u32>) -> Result<(), failure::Error> {
    debug!("grain.rs, delete_grain_images()");

    Ok(())
}

fn list_of_selected_grain_images(user_id: u16, samplename: &str) -> Result<Vec<(String, u32)>, failure::Error> {
    debug!("grain.rs, list_of_selected_grain_images()");

    Ok(Vec::new())
}

fn user_has_image(user_id: u16, samplename: &str, imagename: &str) -> Result<bool, failure::Error> {
    debug!("grain.rs, user_has_image()");

    Ok(false)
}

fn save_outline_for_image(user_id: u16, image_id: u32, image_coordinates: &str, image_axis: &str) -> Result<(), failure::Error> {
    debug!("grain.rs, save_outline_for_image()");

    Ok(())
}

pub fn about_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, about_get()");
    util::show_program(session_id, &ProgramType::Grain3DHe)
}

pub fn load_images_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, load_image_get()");
    if util::logged_in(session_id)? {
        let (user_name, user_db_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_images": list_of_grain_images(user_db_id)?
            });
            // debug!("context: {}", context);
            Ok(Response::html(util::render("grain_load_images", &context)?))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn load_images_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, load_image_post()");
    if util::logged_in(session_id)? {
        let (user_name, user_db_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                image: input::post::BufferedFile,
                sample_name: String,
                size: f64,
                mode: i32,
                mineral: i32,
                ratio_232_238: f64,
                ratio_147_238: f64,
                orientation: i32,
                shape: i32,
                pyramids: i32,
                broken_tips: i32,
                zoned: i32,
                rim_width: f64,
                ratio_rim_core: f64,
            })?;

            let image_input = data.image.filename.ok_or(WebGuiError::NoFilenameForGrainImage)?;
            let image_input = util::replace_characters(&image_input);

            let image_output = match image_input.rfind('.') {
                None => format!("{}.jpg", image_input),
                Some(n) => format!("{}.jpg", image_input[..n].to_string()),
            };

            let samplename = util::replace_characters(&data.sample_name);

            let user_path = format!("user_data/{}/{}", user_name, samplename);

            // debug!("user_path: {}", user_path);

            create_dir_all(&user_path)?;

            let image_path_in = format!("{}/{}", user_path, image_input);
            let image_path_out = format!("{}/{}", user_path, image_output);

            BufWriter::new(File::create(&image_path_in)?).write_all(&data.image.data)?;

            let img_in = image::open(&image_path_in)?;

            let factor : f64 = data.size / 2.0;
            let new_width = ((img_in.width() as f64) * factor) as u32;
            let new_height = ((img_in.height() as f64) * factor) as u32;
            let img_out = image::imageops::resize(&img_in, new_width, new_height, image::FilterType::Nearest);

            img_out.save(image_path_out)?;

            // debug!("mime: {}, in: {}, out: {}, filelength: {}", data.image.mime, image_input, image_output, data.image.data.len());

            add_grain_image(user_db_id, GrainImage {
                id: 0, // Will be created automatically in database
                user_id: user_db_id,
                path: image_output,
                sample_name: samplename,
                size: data.size,
                mode: data.mode,
                mineral: data.mineral,
                ratio_232_238: data.ratio_232_238,
                ratio_147_238: data.ratio_147_238,
                orientation: data.orientation,
                shape: data.shape,
                pyramids: data.pyramids,
                broken_tips: data.broken_tips != 0,
                zoned: data.zoned != 0,
                rim_width: data.rim_width,
                ratio_rim_core: data.ratio_rim_core,
                coordinates: Vec::new(),
                axis: Axis{ x1: 0, y1: 0, x2: 0, y2: 0 },
            })?;

            Ok(Response::redirect_303("/grain/load_images"))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn remove_images_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, remove_image_post()");
    if util::logged_in(session_id)? {
        let (_user_name, user_db_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                remove: Vec<u32>
            })?;

            // debug!("remove: {:?}", data.remove);

            delete_grain_images(user_db_id, data.remove)?;

            Ok(Response::redirect_303("/grain/load_images"))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn outline_images_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, outline_image_get()");
    if util::logged_in(session_id)? {
        let (user_name, user_db_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_db_id)?
            });

            // debug!("context: {}", context);

            Ok(Response::html(util::render("grain_outline_images", &context)?))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn outline_images_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, outline_image_post()");
    if util::logged_in(session_id)? {
        let (user_name, user_db_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                sample: String
            })?;

            let samplename = util::replace_characters(&data.sample);
            let sample_images = list_of_selected_grain_images(user_db_id, &samplename)?.iter().map(
                |(imagename, image_id)| (format!("{}/{}/{}", user_name, samplename, imagename), *image_id) ).collect::<Vec<_>>();

            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_db_id)?,
                "sample_images": sample_images
            });

            // debug!("context: {}", context);

            Ok(Response::html(util::render("grain_outline_images", &context)?))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn sample_image_get(session_id: &str, username: String, samplename: String, imagename: String) -> Result<Response, failure::Error> {
    debug!("grain.rs, sample_image_get()");
    if util::logged_in(session_id)? {
        let (user_name, user_db_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let samplename = util::replace_characters(&samplename);
            let imagename = util::replace_characters(&imagename);

            if username == user_name && user_has_image(user_db_id, &samplename, &imagename)? {
                let filename = format!("user_data/{}/{}/{}", username, samplename, imagename);
                // debug!("image filename: {}", filename);
                let file = File::open(filename)?;
                Ok(Response::from_file("image/jpeg", file))
            } else {
                Err(WebGuiError::GrainImageNotFoundForUser.into())
            }
        } else {
            Err(WebGuiError::ProgramNotAllowedForUser.into())
        }
    } else {
        Err(WebGuiError::UserNotLoggedIn.into())
    }
}

pub fn store_outline_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, store_outline_post()");
    if util::logged_in(session_id)? {
        let (user_name, user_db_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                coordinates: Vec<String>,
                axis: Vec<String>,
                image_ids: Vec<u32>,
            })?;

            // TODO: Save coordinates and axis in database
            for i in 0..(data.coordinates.len()) {
                save_outline_for_image(user_db_id, data.image_ids[i], &data.coordinates[i], &data.axis[i])?;
            }

            // debug!("post data, coordinates: {:?}, axis: {:?}, image_ids: {:?}", data.coordinates, data.axis, data.image_ids);

            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_db_id)?,
                "message": "Outlines and axis saved!"
            });

            // debug!("context: {}", context);

            Ok(Response::html(util::render("grain_outline_images", &context)?))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}
