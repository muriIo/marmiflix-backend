#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("This name is already in queue: \"{0}\"")]
    DuplicateName(String),
    #[error("No queue entry found for id: \"{0}\"")]
    NotFound(String),
    #[error("Session token does not match queue entry: \"{0}\"")]
    Forbidden(String),

    #[error("{0}")]
    WrongPhase(String),
    #[error("{0}")]
    QueueBusy(String),
    #[error("{0}")]
    Validation(String),

    #[error("The queue is full at the moment.")]
    QueueFull(String),
}
