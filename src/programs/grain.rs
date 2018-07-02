use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};

use rouille::{Response, Request, input};
use failure;
use serde::{Serializer};
use image::{self, GenericImage};

use util::{render, show_program, build_program_menu, get_template_name, replace_characters};
use program_types::{ProgramType};
use database::{login_id, logged_in, list_of_allowed_programs, list_of_grain_images, add_grain_image,
    delete_grain_images, list_of_grain_samples, user_has_image, list_of_selected_grain_images};
use error::{WebGuiError};

#[derive(Queryable, PartialEq, Debug, Serialize)]
pub struct GrainImage {
    pub id: i32,
    pub user_id: i32,
    pub path: String,
    pub sample_name: String,
    pub size: f64,
    #[serde(serialize_with = "grain_mode")]
    pub mode: i32,
    #[serde(serialize_with = "grain_mineral")]
    pub mineral: i32,
    pub ratio_232_238: f64,
    pub ratio_147_238: f64,
    #[serde(serialize_with = "grain_orientation")]
    pub orientation: i32,
    #[serde(serialize_with = "grain_shape")]
    pub shape: i32,
    pub pyramids: i32,
    pub broken_tips: bool,
    pub zoned: bool,
    pub rim_width: f64,
    pub ratio_rim_core: f64,
}

fn grain_mode<S>(mode: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    match mode {
        0 => serializer.serialize_str("normal"),
        _ => serializer.serialize_str("cut")
    }
}

fn grain_mineral<S>(mode: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    match mode {
        0 => serializer.serialize_str("ap"),
        _ => serializer.serialize_str("zr")
    }
}

fn grain_orientation<S>(mode: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    match mode {
        0 => serializer.serialize_str("parallel"),
        _ => serializer.serialize_str("perpendicular")
    }
}

fn grain_shape<S>(mode: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    match mode {
        0 => serializer.serialize_str("hexagonal"),
        1 => serializer.serialize_str("ellipsoid"),
        2 => serializer.serialize_str("cylinder"),
        _ => serializer.serialize_str("block")
    }
}

pub fn about_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, about_get()");
    show_program(session_id, &ProgramType::Grain3DHe)
}

pub fn load_images_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, load_image_get()");
    if logged_in(session_id)? {
        let (user_name, user_db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let context = json!({
                "login_id": user_name,
                "programs": build_program_menu(&allowed_programs),
                "grain_images": list_of_grain_images(user_db_id)?
            });
            debug!("context: {}", context);
            Ok(Response::html(render("grain_load_images", &context)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn load_images_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, load_image_post()");
    if logged_in(session_id)? {
        let (user_name, user_db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(user_db_id)?;

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
            let image_input = replace_characters(&image_input);

            let image_output = match image_input.rfind('.') {
                None => format!("{}.jpg", image_input),
                Some(n) => format!("{}.jpg", image_input[..n].to_string()),
            };

            let samplename = replace_characters(&data.sample_name);

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

            debug!("mime: {}, in: {}, out: {}, filelength: {}", data.image.mime, image_input, image_output, data.image.data.len());

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
            })?;

            Ok(Response::redirect_303("/grain/load_images"))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn remove_images_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, remove_image_post()");
    if logged_in(session_id)? {
        let (_user_name, user_db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                remove: Vec<i32>
            })?;

            debug!("remove: {:?}", data.remove);

            delete_grain_images(user_db_id, data.remove)?;

            Ok(Response::redirect_303("/grain/load_images"))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn outline_images_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, outline_image_get()");
    if logged_in(session_id)? {
        let (user_name, user_db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let context = json!({
                "login_id": user_name,
                "programs": build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_db_id)?
            });

            debug!("context: {}", context);

            Ok(Response::html(render("grain_outline_images", &context)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn outline_images_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, outline_image_post()");
    if logged_in(session_id)? {
        let (user_name, user_db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                sample: String
            })?;

            let samplename = replace_characters(&data.sample);
            let sampleimages = list_of_selected_grain_images(user_db_id, &samplename)?.iter().map(
                |imagename| format!("{}/{}/{}", user_name, samplename, imagename)).collect::<Vec<String>>();

            let context = json!({
                "login_id": user_name,
                "programs": build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_db_id)?,
                "sample_images": sampleimages
            });

            Ok(Response::html(render("grain_outline_images", &context)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn sample_image_get(session_id: &str, username: String, samplename: String, imagename: String) -> Result<Response, failure::Error> {
    debug!("grain.rs, sample_image_get()");
    if logged_in(session_id)? {
        let (user_name, user_db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let samplename = replace_characters(&samplename);
            let imagename = replace_characters(&imagename);

            if username == user_name && user_has_image(user_db_id, &samplename, &imagename)? {
                let filename = format!("user_data/{}/{}/{}", username, samplename, imagename);
                debug!("image filename: {}", filename);
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
