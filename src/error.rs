use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Couldn't find specified action in index")]
    CouldNotFindAction,
    #[error("Invalid query provided")]
    InvalidQuery,
    #[error("Action returned an error")]
    ActionReturnedError,
}
