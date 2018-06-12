use serde::{Serialize};
use serde_json;
use handlebars::{Handlebars};
use failure;
use rouille::{Response, Request};


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
        GrainFTCorrection => "grain",
        LandLabESD => "landlab",
        IceCascade => "icecascade",
        CoupledLandscapeThermalSimulator => "coupled",
    }
}

pub fn build_context(login_id: &str, allowed_programs: &Vec<ProgramType>) -> serde_json::Value {
    debug!("util.rs, build_context()");
    json!({"login_id": login_id, "programs": allowed_programs.iter().map(get_template_name).collect::<Vec<_>>()})
}

pub fn show_program(session_id: &str, program: &ProgramType) -> Result<Response, failure::Error> {
    debug!("util.rs, show_program()");

    if logged_in(session_id)? {
        let user_login_id = login_id(session_id)?;
        let allowed_programs = list_of_allowed_programs(&user_login_id)?;

        if allowed_programs.contains(&program) {
            let context = build_context(&user_login_id, &allowed_programs);
            Ok(Response::html(render(get_template_name(program), &context)?))
        } else {
            Ok(Response::redirect_303(get_template_name(&allowed_programs[0])))
        }
    } else {
        Ok(Response::redirect_303("/"))
    }
}
