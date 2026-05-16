use thiserror::Error;

/// All errors produced by Civium core operations.
///
/// Re-exported by `civium_sdk` — match on this when handling [`anyhow::Error`]
/// values downcast from SDK calls:
///
/// ```rust,no_run
/// # use civium_sdk::CiviumError;
/// # fn handle(err: anyhow::Error) {
/// if let Some(e) = err.downcast_ref::<CiviumError>() {
///     match e {
///         CiviumError::Identity(_) => eprintln!("identity problem"),
///         CiviumError::Crypto(_)   => eprintln!("cryptographic error"),
///         _                        => eprintln!("other: {e}"),
///     }
/// }
/// # }
/// ```
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

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("messaging error: {0}")]
    Messaging(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
