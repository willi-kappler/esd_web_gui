use rouille::{Response, Request};
use failure;

use util::{render, show_program, build_program_menu, get_template_name};
use program_types::{ProgramType};
use database::{login_id, logged_in, list_of_allowed_programs, list_of_grain_images, add_grain_image};

#[derive(Queryable, PartialEq, Debug, Serialize)]
pub struct GrainImage {
    pub id: i32,
    pub user_id: i32,
    pub path: String,
    pub sample_name: String,
    pub size: f64,
    pub mode: i32,
    pub mineral: i32,
    pub ratio_232_238: f64,
    pub ratio_147_238: f64,
    pub orientation: i32,
    pub shape: i32,
    pub pyramids: i32,
    pub broken_tips: bool,
    pub zoned: bool,
    pub rim_width: f64,
    pub ratio_rim_core: f64,
}

pub fn about_get(session_id: &str) -> Result<Response, failure::Error> {
    show_program(session_id, &ProgramType::Grain3DHe)
}

pub fn load_images_get(session_id: &str) -> Result<Response, failure::Error> {
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
    if logged_in(session_id)? {
        let (user_name, user_db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(user_db_id)?;

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            let data = post_input!(request, {
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
                broken_tips: i32,
                zoned: i32,
                rim_width: f64,
                ratio_rim_core: f64,
            })?;

            add_grain_image(user_db_id, GrainImage {
                id: 0, // Will be created automatically in database
                user_id: user_db_id,
                path: data.path,
                sample_name: data.sample_name,
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
