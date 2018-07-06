use std::sync::{Mutex, MutexGuard};
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, BufReader, Write, Read};

use rouille::{Response, Request, input};
use failure;
use image::{self, GenericImage};
use toml;
use itertools::Itertools;
use serde_json;

use util;
use configuration;
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
    file_name: String,
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

fn get_db_lock<'a>() -> Result<MutexGuard<'a, Vec<GrainImage>>, failure::Error> {
    GRAIN_DB.lock().map_err(|_| WebGuiError::GrainDBMutexLockError.into())
}

pub fn load_db() -> Result<(), failure::Error> {
    debug!("utils.rs, load_db()");
    let mut grain_db = get_db_lock()?;

    let mut data = String::new();
    let f = File::open(configuration::grain_db()?)?;
    let mut f = BufReader::new(f);
    f.read_to_string(&mut data)?;

    *grain_db = toml::from_str(&data)?;
    Ok(())
}

fn save_db() -> Result<(), failure::Error> {
    debug!("utils.rs, load_db()");
    let grain_db = get_db_lock()?;

    let serialized = toml::Value::try_from(&*grain_db)?.to_string();
    let f = File::create(configuration::grain_db()?)?;
    let mut f = BufWriter::new(f);
    f.write_all(serialized.as_bytes())?;

    Ok(())
}

fn list_of_grain_images(user_id: u16) -> Result<Vec<GrainImage>, failure::Error> {
    debug!("grain.rs, list_of_grain_images()");
    let grain_db = get_db_lock()?;

    Ok(grain_db.iter()
        .filter(|grain| grain.user_id == user_id)
        .map(|grain| grain.clone()).collect())
}

fn list_of_grain_samples(user_id: u16) -> Result<Vec<String>, failure::Error> {
    debug!("grain.rs, list_of_grain_samples()");
    let grain_db = get_db_lock()?;

    Ok(grain_db.iter()
        .filter(|grain| grain.user_id == user_id)
        .map(|grain| grain.sample_name.clone())
        .unique().collect())
}

fn create_new_id() -> Result<u32, failure::Error> {
    debug!("grain.rs, create_new_id()");
    let grain_db = get_db_lock()?;

    let num_of_elements = grain_db.len();

    if num_of_elements == 0 {
        Ok(0)
    } else {
        Ok(grain_db[num_of_elements - 1].id + 1)
    }
}

fn add_grain_image(new_image: GrainImage) -> Result<(), failure::Error> {
    debug!("grain.rs, add_grain_images()");
    let mut grain_db = get_db_lock()?;

    grain_db.push(new_image);

    save_db()?;

    Ok(())
}

fn delete_grain_images(user_id: u16, image_ids: Vec<u32>) -> Result<(), failure::Error> {
    debug!("grain.rs, delete_grain_images()");
    let mut grain_db = get_db_lock()?;

    for id in image_ids {
        if let Some(index) = grain_db.iter().position(|grain| grain.id == id && grain.user_id == user_id) {
            grain_db.remove(index);
        }
    }

    save_db()?;

    Ok(())
}

fn list_of_selected_grain_images(user_id: u16, sample_name: &str) -> Result<Vec<(String, u32)>, failure::Error> {
    debug!("grain.rs, list_of_selected_grain_images()");
    let grain_db = get_db_lock()?;

    Ok(grain_db.iter()
        .filter(|grain| grain.user_id == user_id && grain.sample_name == sample_name)
        .map(|grain| (grain.file_name.clone(), grain.id)).collect::<Vec<_>>())
}

fn user_has_image(user_id: u16, sample_name: &str, file_name: &str) -> Result<bool, failure::Error> {
    debug!("grain.rs, user_has_image()");
    let grain_db = get_db_lock()?;

    Ok(grain_db.iter()
    .any(|grain| grain.user_id == user_id && grain.sample_name == sample_name && grain.file_name == file_name))
}

fn save_outline_for_image(user_id: u16, id: u32, coordinates: Vec<(u32, u32)>, axis: Axis) -> Result<(), failure::Error> {
    debug!("grain.rs, save_outline_for_image()");
    let mut grain_db = get_db_lock()?;

    if let Some(index) = grain_db.iter().position(|grain| grain.id == id && grain.user_id == user_id) {
        grain_db[index].coordinates = coordinates;
        grain_db[index].axis = axis;
    }

    save_db()?;

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

            let sample_name = util::replace_characters(&data.sample_name);

            let user_path = format!("user_data/{}/{}", user_name, sample_name);

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

            add_grain_image(GrainImage {
                id: create_new_id()?,
                user_id: user_db_id,
                file_name: image_output,
                sample_name: sample_name,
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
                let coordinates: Vec<(u32, u32)> = serde_json::from_str(&data.coordinates[i])?;
                let axis: Axis = serde_json::from_str(&data.axis[i])?;
                save_outline_for_image(user_db_id, data.image_ids[i], coordinates, axis)?;
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
