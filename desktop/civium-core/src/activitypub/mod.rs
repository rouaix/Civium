use serde::{Deserialize, Serialize};

/// État de la fédération ActivityPub pour un réseau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApStatus {
    pub enabled: bool,
    pub actor_url: Option<String>,
    pub followers_count: usize,
}

/// Un abonné ActivityPub distant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApFollower {
    pub actor_url: String,
    pub inbox_url: String,
    pub shared_inbox: Option<String>,
    pub followed_at: u64,
}

/// Un message publié sur ActivityPub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApPost {
    pub id: i64,
    pub network_cid: String,
    pub note_id: String,
    pub content: String,
    pub ap_activity_id: Option<String>,
    pub posted_at: u64,
}

/// Résultat d'un post ActivityPub (retourné par la commande Tauri).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApPostResult {
    pub note_id: String,
    pub actor_url: String,
    pub delivered_to: u32,
}
