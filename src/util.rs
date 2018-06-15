use serde::{Serialize};
use handlebars::{Handlebars};
use failure;
use rouille::{Response};


use program_types::{ProgramType};
use database::{list_of_allowed_programs, login_id, logged_in};

lazy_static! {
    static ref TEMPLATE : Handlebars = {
        let mut hb = Handlebars::new();
        hb.register_template_file("login", "html/login.hbs").unwrap();
        hb.register_template_file("header", "html/header.hbs").unwrap();
        hb.register_template_file("footer", "html/footer.hbs").unwrap();
        hb.register_template_file("pecube", "html/pecube.hbs").unwrap();
        hb.register_template_file("grain", "html/grain.hbs").unwrap();
        hb.register_template_file("grain_load_images", "html/grain_load_images.hbs").unwrap();
        hb.register_template_file("grain_outline_images", "html/grain_outline_images.hbs").unwrap();
        hb.register_template_file("landlab", "html/landlab.hbs").unwrap();
        hb.register_template_file("icecascade", "html/icecascade.hbs").unwrap();
        hb.register_template_file("coupled", "html/coupled.hbs").unwrap();
        hb
    };
}

pub fn render<T: Serialize>(name: &str, context: &T) -> Result<String, failure::Error> {
    debug!("util.rs, render()");
    TEMPLATE.render(name, context).map_err(|e| e.into())
}

pub fn get_template_name<'a>(program: &ProgramType) -> &'a str {
    debug!("util.rs, get_template_name()");
    use self::ProgramType::*;

    match program {
        PecubeESD => "pecube",
        Grain3DHe => "grain",
        LandLabESD => "landlab",
        IceCascade => "icecascade",
        CoupledLandscapeThermalSimulator => "coupled",
    }
}

pub fn get_menu_name<'a>(program: &ProgramType) -> &'a str {
    debug!("util.rs, get_menu_name()");
    use self::ProgramType::*;

    match program {
        PecubeESD => "PecubeESD",
        Grain3DHe => "3D-He",
        LandLabESD => "LandLab",
        IceCascade => "IceCascade",
        CoupledLandscapeThermalSimulator => "CoupledLandscape",
    }
}

pub fn build_program_menu(allowed_programs: &Vec<ProgramType>) -> Vec<(&str, &str)> {
    allowed_programs.iter().map(|p| (get_template_name(p), get_menu_name(p))).collect::<Vec<_>>()
}

pub fn show_program(session_id: &str, program: &ProgramType) -> Result<Response, failure::Error> {
    debug!("util.rs, show_program()");

    if logged_in(session_id)? {
        let (user_name, db_id) = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(db_id)?;

        if allowed_programs.contains(&program) {
            let user_menu = json!({"login_id": user_name, "programs": build_program_menu(&allowed_programs)});
            debug!("user_menu: {}", user_menu);
            Ok(Response::html(render(get_template_name(program), &user_menu)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}
