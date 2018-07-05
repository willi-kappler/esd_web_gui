use rouille::{Response};
use failure;

use util;

pub fn handle(session_id: &str) -> Result<Response, failure::Error> {
    debug!("logout.rs, handle()");

    if util::logged_in(session_id)? {
        util::logout(session_id)?;
    }

    Ok(Response::redirect_303("/"))
}
