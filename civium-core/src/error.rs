use thiserror::Error;

#[derive(Debug, Error)]
pub enum CiviumError {
    #[error("identity error: {0}")]
    Identity(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("invitation error: {0}")]
    Invitation(String),

    #[error("node error: {0}")]
    Node(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
