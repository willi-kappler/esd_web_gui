use rouille::{Response};
use failure;

use util::{show_program};
use program_types::{ProgramType};

pub fn about_get(session_id: &str) -> Result<Response, failure::Error> {
    show_program(session_id, &ProgramType::IceCascade)
}
