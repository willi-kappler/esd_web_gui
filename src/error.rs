#[derive(Debug, Fail)]
pub enum WebGuiError {
    #[fail(display = "Could not lock CONFIGURATION")]
    ConfigurationMutexLockError,
    #[fail(display = "Could not lock DB_CONNECTION")]
    DatabaseMutexLockError,
    #[fail(display = "Invalid number of command line arguments")]
    InvalidCommandLineArguments,
    #[fail(display = "User name not found")]
    UserNotFound,
    #[fail(display = "Session id not found")]
    SessionNotFound,
    #[fail(display = "Multiple user found with same name")]
    MultipleUsers,
    #[fail(display = "Multiple sessions found with same id")]
    MultipleSessions,
    #[fail(display = "Error while updateing the database")]
    UpdateDBError,
}
