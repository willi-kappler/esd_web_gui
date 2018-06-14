use rouille::{Response, Request};
use failure;
use diesel;

use util::{render, show_program, build_user_menu, get_template_name};
use program_types::{ProgramType};
use database::{login_id, logged_in, list_of_allowed_programs};

#[derive(PartialEq, Debug)]
pub enum GrainMode {
    Normal,
    Cut,
}

#[derive(PartialEq, Debug)]
pub enum GrainMineral {
    Apatite,
    Zircon,
}

#[derive(PartialEq, Debug)]
pub enum GrainOrientation {
    Parallel,
    Perpendicular,
}

#[derive(PartialEq, Debug)]
pub enum GrainShape {
    Hexagonal,
    Ellipsoid,
    Cylinder,
    Block,
}

#[derive(Queryable, PartialEq, Debug)]
pub struct GrainImage {
    pub id: i32,
    pub user_id: i32,
    pub path: String,
    pub sample_name: String,
    pub size: f64,
    pub mode: GrainMode,
    pub mineral: GrainMineral,
    pub ratio_232_238: f64,
    pub ratio_147_238: f64,
    pub orientation: GrainOrientation,
    pub shape: GrainShape,
    pub pyramids: i32,
    pub broken_tips: bool,
    pub zoned: bool,
    pub rim_width: f64,
    pub ratio_rim_core: f64,
}

impl diesel::Queryable<diesel::sql_types::Integer, diesel::sqlite::Sqlite> for GrainMode {
    type Row = i32;

    fn build(row: Self::Row) -> Self {
        match row {
            0 => GrainMode::Normal,
            _ => GrainMode::Cut
        }
    }
}

impl diesel::Expression for GrainMode {
    type SqlType = diesel::sql_types::Integer;
}

impl diesel::Queryable<diesel::sql_types::Integer, diesel::sqlite::Sqlite> for GrainMineral {
    type Row = i32;

    fn build(row: Self::Row) -> Self {
        match row {
            0 => GrainMineral::Apatite,
            _ => GrainMineral::Zircon
        }
    }
}

impl diesel::Expression for GrainMineral {
    type SqlType = diesel::sql_types::Integer;
}

impl diesel::Queryable<diesel::sql_types::Integer, diesel::sqlite::Sqlite> for GrainOrientation {
    type Row = i32;

    fn build(row: Self::Row) -> Self {
        match row {
            0 => GrainOrientation::Parallel,
            _ => GrainOrientation::Perpendicular
        }
    }
}

impl diesel::Expression for GrainOrientation {
    type SqlType = diesel::sql_types::Integer;
}

impl diesel::Queryable<diesel::sql_types::Integer, diesel::sqlite::Sqlite> for GrainShape {
    type Row = i32;

    fn build(row: Self::Row) -> Self {
        match row {
            0 => GrainShape::Hexagonal,
            1 => GrainShape::Ellipsoid,
            2 => GrainShape::Cylinder,
            _ => GrainShape::Block,
        }
    }
}

impl diesel::Expression for GrainShape {
    type SqlType = diesel::sql_types::Integer;
}

pub fn about_get(session_id: &str) -> Result<Response, failure::Error> {
    show_program(session_id, &ProgramType::Grain3DHe)
}

pub fn load_images_get(session_id: &str) -> Result<Response, failure::Error> {
    if logged_in(session_id)? {
        let (user_name, db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(db_id)?;
        let context = build_user_menu(&user_name, &allowed_programs);

        if allowed_programs.contains(&ProgramType::Grain3DHe) {
            Ok(Response::html(render("grain_load_images", &context)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}

pub fn load_images_post(session_id: &str, _request: &Request) -> Result<Response, failure::Error> {
    // TODO
    let (_user_name, _db_id) = login_id(session_id)?;
    Ok(Response::redirect_303("/grain/load_images"))
}
