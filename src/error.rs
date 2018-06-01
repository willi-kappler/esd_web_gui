#[derive(Debug, Fail)]
pub enum WebGuiError {
    #[fail(display = "Login not found for session: {}", session_id)]
    LoginNotFound {
        session_id: String,
    },
    #[fail(display = "Already logged in: session: {}, login1: {}, login2: {}", session_id, login_id, login_id2)]
    AlreadyLoggedIn {
        session_id: String,
        login_id: String,
        login_id2: String,
    },
    #[fail(display = "Could not log out session: {}", session_id)]
    CouldNotLogout {
        session_id: String,
    },
    #[fail(display = "Could not lock USERDATA")]
    UserDataMutexLockError,
}
