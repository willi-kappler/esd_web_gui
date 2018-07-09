#[derive(Debug, Fail)]
pub enum WebGuiError {
    #[fail(display = "Could not lock CONFIGURATION")]
    ConfigurationMutexLockError,
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
    #[fail(display = "Unknown program type")]
    UnknownProgramType,
    #[fail(display = "No programs for user defined in database")]
    NoProgramsForUser,
    #[fail(display = "No filename for grain image found")]
    NoFilenameForGrainImage,
    #[fail(display = "Grain / sample image not found for user")]
    GrainImageNotFoundForUser,
    #[fail(display = "User is not allowed to use that program")]
    ProgramNotAllowedForUser,
    #[fail(display = "User in not logged in")]
    UserNotLoggedIn,
}
