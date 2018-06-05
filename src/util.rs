use std::convert::From;

use serde::{Serialize};
use handlebars::{Handlebars};
use failure;

use error::{WebGuiError};

lazy_static! {
    static ref TEMPLATE : Handlebars = {
        let mut hb = Handlebars::new();
        hb.register_template_file("login", "html/login.hbs").unwrap();
        hb.register_template_file("logout", "html/logout.hbs").unwrap();
        hb.register_template_file("menu", "html/menu.hbs").unwrap();
        hb
    };
}

pub fn render<T: Serialize>(name: &str, context: &T) -> Result<String, failure::Error> {
    debug!("util.rs, render()");
    TEMPLATE.render(name, context).map_err(From::from)
}
