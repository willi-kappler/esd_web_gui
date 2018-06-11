use rouille::{Response, Request};
use failure;

use util::{render, show_program};
use programs::{ProgramType};
use database::{login_id};

pub fn handle_get(session_id: &str) -> Result<Response, failure::Error> {
    show_program(session_id, &ProgramType::LandLabESD)
}

pub fn handle_post(session_id: &str, _request: &Request) -> Result<Response, failure::Error> {
    // TODO
    let _user_login_id = login_id(session_id)?;
    Ok(Response::redirect_303("menu_landlab"))
}
