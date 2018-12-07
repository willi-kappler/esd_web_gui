use std::sync::{Mutex, MutexGuard};
use std::fs::{create_dir_all, remove_file, File};
use std::io::{BufWriter, BufReader, Write, Read};
use std::path::Path;
use std::collections::HashSet;
use std::{thread, time, env};
use std::process::Command;

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
struct GrainList {
    grains: Vec<GrainImage>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Coordinates {
    x: u32,
    y: u32,
}

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
    coordinates: Vec<Coordinates>,
    coordinate_file_name: String,
    axis: Axis,
}

lazy_static! {
    static ref GRAIN_DB : Mutex<Vec<GrainImage>> = {
        Mutex::new(Vec::new())
    };
}

fn get_db_lock<'a>() -> MutexGuard<'a, Vec<GrainImage>> {
    loop {
        let lock = GRAIN_DB.try_lock();
        if let Ok(mutex) = lock {
            return mutex
        } else {
            debug!("grain.rs, get_db_lock() -> thread_sleep");
            // Sleep and try again to acquire the lock
            let duration = time::Duration::from_millis(100);
            thread::sleep(duration);
        }
    }
}

pub fn load_db() -> Result<(), failure::Error> {
    debug!("utils.rs, load_db()");
    let mut grain_db = get_db_lock();

    let mut data = String::new();
    let f = File::open(configuration::grain_db())?;
    let mut f = BufReader::new(f);
    f.read_to_string(&mut data)?;

    let grain_list: GrainList = toml::from_str(&data)?;
    *grain_db = grain_list.grains;
    Ok(())
}

fn save_db(grain_db: &Vec<GrainImage>) -> Result<(), failure::Error> {
    debug!("utils.rs, save_db()");
    let grain_list = GrainList{ grains: grain_db.clone() };

    let serialized = toml::Value::try_from(grain_list)?.to_string();
    let f = File::create(configuration::grain_db())?;
    let mut f = BufWriter::new(f);
    f.write_all(serialized.as_bytes())?;

    Ok(())
}

fn list_of_grain_images(user_id: u16) -> Result<Vec<GrainImage>, failure::Error> {
    debug!("grain.rs, list_of_grain_images()");
    let grain_db = get_db_lock();

    Ok(grain_db.iter()
        .filter(|grain| grain.user_id == user_id)
        .map(|grain| grain.clone()).collect())
}

fn list_of_grain_samples(user_id: u16) -> Result<Vec<String>, failure::Error> {
    debug!("grain.rs, list_of_grain_samples()");
    let grain_db = get_db_lock();

    Ok(grain_db.iter()
        .filter(|grain| grain.user_id == user_id)
        .map(|grain| grain.sample_name.clone())
        .unique().collect())
}

fn create_new_id() -> Result<u32, failure::Error> {
    debug!("grain.rs, create_new_id()");
    let grain_db = get_db_lock();

    let num_of_elements = grain_db.len();

    if num_of_elements == 0 {
        Ok(0)
    } else {
        Ok(grain_db[num_of_elements - 1].id + 1)
    }
}

fn add_grain_image(new_image: GrainImage) -> Result<(), failure::Error> {
    debug!("grain.rs, add_grain_images()");
    let mut grain_db = get_db_lock();
    grain_db.push(new_image);

    save_db(&*grain_db)?;

    Ok(())
}

fn delete_grain_images(user_id: u16, image_ids: Vec<u32>) -> Result<(), failure::Error> {
    debug!("grain.rs, delete_grain_images()");
    let mut grain_db = get_db_lock();

    for id in image_ids {
        if let Some(index) = grain_db.iter().position(|grain| grain.id == id && grain.user_id == user_id) {
            grain_db.remove(index);
        }
    }

    save_db(&*grain_db)?;

    Ok(())
}

fn list_of_selected_grain_images(user_id: u16, sample_name: &str) -> Result<Vec<(String, u32)>, failure::Error> {
    debug!("grain.rs, list_of_selected_grain_images()");
    let grain_db = get_db_lock();

    Ok(grain_db.iter()
        .filter(|grain| grain.user_id == user_id && grain.sample_name == sample_name)
        .map(|grain| (grain.file_name.clone(), grain.id)).collect::<Vec<_>>())
}

fn user_has_image(user_id: u16, sample_name: &str, file_name: &str) -> Result<bool, failure::Error> {
    debug!("grain.rs, user_has_image()");
    let grain_db = get_db_lock();

    Ok(grain_db.iter()
    .any(|grain| grain.user_id == user_id && grain.sample_name == sample_name && grain.file_name == file_name))
}

fn save_outline_for_image(user_id: u16, id: u32, coordinates: Vec<Coordinates>, axis: Axis) -> Result<(), failure::Error> {
    debug!("grain.rs, save_outline_for_image()");
    let mut grain_db = get_db_lock();

    if let Some(index) = grain_db.iter().position(|grain| grain.id == id && grain.user_id == user_id) {
        grain_db[index].coordinates = coordinates;
        grain_db[index].axis = axis;
    }

    save_db(&*grain_db)?;

    Ok(())
}

fn submit_calculation(user_id: u16, user_name: &str, sample_name: &str) -> Result<(), failure::Error> {
    debug!("grain.rs, submit_calculation()");
    let mut grain_db = get_db_lock();

    let grain_folder = format!("matlab/{}/{}", user_name, sample_name);
    create_dir_all(&grain_folder)?;

    let input_file = format!("{}/matlab_input.csv", grain_folder);
    let f = File::create(&input_file)?;
    let mut grain_file = BufWriter::new(f);

    // Write out header
    write!(grain_file, "# coordinate file, sample name, size, mode, mineral, ratio 232-238, ratio 147-238, orientation, shape, pyramids, broken tips, zoned, rim width, ratio rim core, axis x1, axis y1, axis x2, axis y2\n")?;

    for grain in grain_db.iter_mut() {
        if grain.user_id == user_id && grain.sample_name == sample_name {
            write!(grain_file, "{}, ", grain.coordinate_file_name)?;
            write!(grain_file, "{}, ", grain.sample_name)?;
            write!(grain_file, "{}, ", grain.size)?;
            write!(grain_file, "{}, ", if grain.mode == 0 {"normal"} else {"cut"})?;
            write!(grain_file, "{}, ", if grain.mineral == 0 {"ap"} else {"zr"})?;
            write!(grain_file, "{}, ", grain.ratio_232_238)?;
            write!(grain_file, "{}, ", grain.ratio_147_238)?;
            write!(grain_file, "{}, ", if grain.orientation == 0 {"parallel"} else {"perpendicular"})?;
            write!(grain_file, "{}, ",
                match grain.shape {
                    0 => "hexagonal",
                    1 => "ellipsoid",
                    2 => "cylinder",
                    3 => "block",
                    _ => "unknown"
                }
            )?;
            write!(grain_file, "{}, ", grain.pyramids)?;
            write!(grain_file, "{}, ", grain.broken_tips)?;
            write!(grain_file, "{}, ", grain.zoned)?;
            write!(grain_file, "{}, ", grain.rim_width)?;
            write!(grain_file, "{}, ", grain.ratio_rim_core)?;
            write!(grain_file, "{}, ", grain.axis.x1)?;
            write!(grain_file, "{}, ", grain.axis.y1)?;
            write!(grain_file, "{}, ", grain.axis.x2)?;
            write!(grain_file, "{}\n", grain.axis.y2)?;

            let f = File::create(format!("{}/{}", grain_folder, grain.coordinate_file_name))?;
            let mut coordinates_file = BufWriter::new(f);

            for coordinate in grain.coordinates.iter() {
                write!(coordinates_file, "{}, {}\n", coordinate.x, coordinate.y)?;
            }
        }
    }

    let output_file = format!("{}/result.txt", grain_folder);
    if Path::new(&output_file).exists() {
        remove_file(&output_file)?;
    }

    let current_folder = env::current_dir().unwrap();
    let script_start = format!("input_file='{}';output_file='{}';grain_folder='{}/{}';run('run_3DFt.m')", input_file, output_file, current_folder.display(), grain_folder);

    Command::new(configuration::matlab_exec())
        .args(&["-nodisplay", "-nosplash", "-nodesktop", "-sd", &configuration::matlab_folder(), "-r", &script_start])
        .spawn()?;

    Ok(())

/*
    Test on MacOS:
            /Applications/MATLAB_R2018a.app/bin/matlab -nodisplay -nosplash -nodesktop -sd /Users/willi/tmp/FT_model_180419 -r "input_file='matlab_input.csv';output_file='result.txt';grain_folder='/Users/willi/tmp/git/web_gui/matlab/test_user/test1';run('run_3DFt.m')"

    Test on Linux (webserver):
*/

}

fn get_results(user_id: u16, user_name: &str) -> Result<Vec<(String, String)>, failure::Error> {
    debug!("grain.rs, get_results()");
    let mut grain_db = get_db_lock();

    let mut results = Vec::new();
    let mut already_processed = HashSet::new();

    for grain in grain_db.iter_mut() {
        if grain.user_id == user_id {
            let path = format!("matlab/{}/{}/result.txt", user_name, grain.sample_name);
            if Path::new(&path).exists() {
                if  !already_processed.contains(&grain.sample_name) {
                    let mut f = File::open(path)?;
                    let mut contents = String::new();
                    f.read_to_string(&mut contents)?;

                    results.push((grain.sample_name.clone(), contents));

                    already_processed.insert(grain.sample_name.clone());
                }
            }
        }
    }

    Ok(results)
}



// URL route targets:

pub fn about_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, about_get()");
    util::show_program(session_id, &ProgramType::Grain3DHe)
}

pub fn load_images_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, load_image_get()");
    if util::logged_in(session_id)? {
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_images": list_of_grain_images(user_id)?
            });
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
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

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

            let coordinate_file_name = image_output.replace(".jpg", ".txt");

            let sample_name = util::replace_characters(&data.sample_name);

            let user_path = format!("user_data/{}/{}", user_name, sample_name);

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

            add_grain_image(GrainImage {
                id: create_new_id()?,
                user_id: user_id,
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
                coordinate_file_name,
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
        let (_user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                remove: Vec<u32>
            })?;

            delete_grain_images(user_id, data.remove)?;

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
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_id)?
            });

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
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                sample: String
            })?;

            let samplename = util::replace_characters(&data.sample);
            let sample_images = list_of_selected_grain_images(user_id, &samplename)?.iter().map(
                |(imagename, image_id)| (format!("{}/{}/{}", user_name, samplename, imagename), *image_id) ).collect::<Vec<_>>();

            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_id)?,
                "sample_images": sample_images
            });

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
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let samplename = util::replace_characters(&samplename);
            let imagename = util::replace_characters(&imagename);

            if username == user_name && user_has_image(user_id, &samplename, &imagename)? {
                let filename = format!("user_data/{}/{}/{}", username, samplename, imagename);
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
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                coordinates: Vec<String>,
                axis: Vec<String>,
                image_ids: Vec<u32>,
            })?;

            for i in 0..(data.coordinates.len()) {
                let coordinates: Vec<Coordinates> = serde_json::from_str(&data.coordinates[i])?;
                let axis: Axis = serde_json::from_str(&data.axis[i])?;
                save_outline_for_image(user_id, data.image_ids[i], coordinates, axis)?;
            }

            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_id)?,
                "message": "Outlines and axis saved!"
            });

            Ok(Response::html(util::render("grain_outline_images", &context)?))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn calculate_get(session_id: &str) -> Result<Response, failure::Error> {
    debug!("grain.rs, calculate_get()");
    if util::logged_in(session_id)? {
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let results = get_results(user_id, &user_name)?;

            debug!("results: {:?}", results);

            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_id)?,
                "message": if results.len() == 0 {"No results yet"} else {""},
                "results": results,
            });

            Ok(Response::html(util::render("grain_calculate", &context)?))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn calculate_post(session_id: &str, request: &Request) -> Result<Response, failure::Error> {
    debug!("grain.rs, calculate_post()");
    if util::logged_in(session_id)? {
        let (user_name, user_id) = util::login_id(session_id)?;
        let allowed_programs = util::list_of_allowed_programs(user_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
                sample: String
            })?;

            let sample_name = util::replace_characters(&data.sample);
            submit_calculation(user_id, &user_name,  &sample_name)?;

            let context = json!({
                "login_id": user_name,
                "programs": util::build_program_menu(&allowed_programs),
                "grain_samples": list_of_grain_samples(user_id)?,
                "message": "Calculation submitted!",
            });

            Ok(Response::html(util::render("grain_calculate", &context)?))
        } else {
            Ok(Response::redirect_303(util::get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}
