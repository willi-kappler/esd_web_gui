use serde::{Serialize};
use handlebars::{Handlebars};
use failure;

use programs::{ProgramType};
use database::{login_id, list_of_allowed_programs};

lazy_static! {
    static ref TEMPLATE : Handlebars = {
        let mut hb = Handlebars::new();
        hb.register_template_file("login", "html/login.hbs").unwrap();
        hb.register_template_file("logout", "html/logout.hbs").unwrap();
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

fn get_template_name<'a>(program: &ProgramType) -> &'a str {
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

pub fn show_program(login_id: &str, selected: Option<ProgramType>) -> Result<String, failure::Error> {
    debug!("util.rs, show_program()");
    let allowed_programs = list_of_allowed_programs(login_id)?;
    let context = &json!({"login_id": login_id, "programs":
        allowed_programs.iter().map(get_template_name).collect::<Vec<&str>>()});

    match selected {
        Some(program) if allowed_programs.contains(&program) => {
            render(get_template_name(&program), context)
        }
        _ => {
            render(get_template_name(&allowed_programs[0]), context)
        }
    }
}
