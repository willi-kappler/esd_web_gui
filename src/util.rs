use handlebars::{Handlebars};

lazy_static! {
    pub static ref TEMPLATE : Handlebars = {
        let mut hb = Handlebars::new();
        hb.register_template_file("login", "html/login.hbs").unwrap();
        hb.register_template_file("logout", "html/logout.hbs").unwrap();
        hb.register_template_file("main", "html/main.hbs").unwrap();
        hb
    };
}

pub fn logged_in(client_id: &str) -> bool {
    true
}

pub fn check_login(login_id: &str, password: &str) -> bool {
    true
}
