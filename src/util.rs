use serde::{Serialize};
use serde_json;
use handlebars::{Handlebars};
use failure;
use rouille::{Response, Request};


use programs::{ProgramType};
use database::{list_of_allowed_programs, login_id};

lazy_static! {
    static ref TEMPLATE : Handlebars = {
        let mut hb = Handlebars::new();
        hb.register_template_file("login", "html/login.hbs").unwrap();
        hb.register_template_file("header", "html/header.hbs").unwrap();
        hb.register_template_file("footer", "html/footer.hbs").unwrap();
        hb.register_template_file("menu_pecube", "html/menu_pecube.hbs").unwrap();
        hb.register_template_file("menu_grain", "html/menu_grain.hbs").unwrap();
        hb.register_template_file("menu_landlab", "html/menu_landlab.hbs").unwrap();
        hb.register_template_file("menu_icecascade", "html/menu_icecascade.hbs").unwrap();
        hb.register_template_file("menu_coupled", "html/menu_coupled.hbs").unwrap();
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
        PecubeESD => "menu_pecube",
        GrainFTCorrection => "menu_grain",
        LandLabESD => "menu_landlab",
        IceCascade => "menu_icecascade",
        CoupledLandscapeThermalSimulator => "menu_coupled",
    }
}

pub fn build_context(login_id: &str, allowed_programs: &Vec<ProgramType>) -> serde_json::Value {
    debug!("util.rs, build_context()");
    json!({"login_id": login_id, "programs": allowed_programs.iter().map(get_template_name).collect::<Vec<_>>()})
}

pub fn show_program(session_id: &str, program: &ProgramType) -> Result<Response, failure::Error> {
    debug!("util.rs, show_program()");

    let user_login_id = login_id(session_id)?;
    let allowed_programs = list_of_allowed_programs(&user_login_id)?;

    if allowed_programs.contains(&program) {
        let context = build_context(&user_login_id, &allowed_programs);
        Ok(Response::html(render(get_template_name(program), &context)?))
    } else {
        Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
    }
}
