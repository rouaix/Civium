use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A calendar event belonging to a Civium network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaEvent {
    pub id: String,
    pub network_cid_short: String,
    pub title: String,
    pub description: String,
    pub start_at: u64,
    pub end_at: Option<u64>,
    pub recurrence: Option<String>,
    pub location: Option<String>,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl AgendaEvent {
    pub fn new(
        network_cid_short: String,
        title: String,
        description: String,
        start_at: u64,
        end_at: Option<u64>,
        location: Option<String>,
        created_by: String,
    ) -> Self {
        let now = unix_now();
        Self {
            id: uuid(),
            network_cid_short,
            title,
            description,
            start_at,
            end_at,
            recurrence: None,
            location,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }
fn uuid() -> String { Uuid::new_v4().to_string() }
