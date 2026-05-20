use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
pub mod sandbox;

/// Lifecycle state of an installed plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Enabled,
    Disabled,
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Enabled  => f.write_str("enabled"),
            Self::Disabled => f.write_str("disabled"),
        }
    }
}

impl std::str::FromStr for PluginState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enabled"  => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(format!("unknown plugin state '{s}'")),
        }
    }
}

/// Data capability a plugin must declare to use a CIL action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginPermission {
    ReadMembers,
    ReadMessages,
    WriteMessages,
    ReadGovernance,
    WriteGovernance,
    ReadDirectory,
    WriteDirectory,
    ReadConnections,
    ReadAgenda,
    WriteAgenda,
    ReadDocuments,
    WriteDocuments,
}

impl std::fmt::Display for PluginPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ReadMembers     => "read:members",
            Self::ReadMessages    => "read:messages",
            Self::WriteMessages   => "write:messages",
            Self::ReadGovernance  => "read:governance",
            Self::WriteGovernance => "write:governance",
            Self::ReadDirectory   => "read:directory",
            Self::WriteDirectory  => "write:directory",
            Self::ReadConnections => "read:connections",
            Self::ReadAgenda      => "read:agenda",
            Self::WriteAgenda     => "write:agenda",
            Self::ReadDocuments   => "read:documents",
            Self::WriteDocuments  => "write:documents",
        };
        f.write_str(s)
    }
}

/// Certification level of a plugin in the Civium ecosystem.
///
/// Levels are cumulative: each builds on the requirements of the previous one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CertificationLevel {
    /// No certification — use at your own risk.
    #[default]
    Uncertified,
    /// Basic certification: open-source, documented, passes automated scans.
    Minimal,
    /// Listed in the Registre de Services Civium (RSC): verified publisher + peer review.
    Rsc,
    /// Full certification: external security audit, formal code review by the Civium team.
    Certified,
}

impl std::fmt::Display for CertificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uncertified => f.write_str("uncertified"),
            Self::Minimal     => f.write_str("minimal"),
            Self::Rsc         => f.write_str("rsc"),
            Self::Certified   => f.write_str("certified"),
        }
    }
}

/// Static metadata declared by a plugin author.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub permissions: Vec<PluginPermission>,
    /// True for system plugins (Gouvernance, CIL) that cannot be disabled.
    #[serde(default)]
    pub is_system: bool,
    /// Certification level in the Civium ecosystem.
    #[serde(default)]
    pub certification: CertificationLevel,
}

/// Runtime record persisted in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRecord {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub installed_at: u64,
}

impl PluginRecord {
    pub fn new(manifest: PluginManifest) -> Self {
        // System plugins start enabled; others start disabled until explicitly enabled.
        let state = if manifest.is_system { PluginState::Enabled } else { PluginState::Disabled };
        Self { state, manifest, installed_at: unix_now() }
    }

    pub fn new_enabled(manifest: PluginManifest) -> Self {
        Self { state: PluginState::Enabled, manifest, installed_at: unix_now() }
    }
}

fn unix_now() -> u64 { crate::time::unix_now() }

/// Returns the manifests for all pre-installed Civium plugins.
pub fn preinstalled_plugins() -> Vec<(PluginManifest, bool)> {
    // (manifest, enabled_by_default)
    vec![
        (PluginManifest {
            id: "civium.gouvernance".to_string(),
            name: "Gouvernance".to_string(),
            version: "1.0.0".to_string(),
            description: "Propositions, votes, délégations, garde-fou majoritaire.".to_string(),
            author: "Civium".to_string(),
            permissions: vec![
                PluginPermission::ReadGovernance,
                PluginPermission::WriteGovernance,
                PluginPermission::ReadMembers,
            ],
            is_system: true,
            certification: CertificationLevel::Certified,
        }, true),
        (PluginManifest {
            id: "civium.cil".to_string(),
            name: "CIL (Civium Integration Layer)".to_string(),
            version: "1.0.0".to_string(),
            description: "Couche de contrôle d'accès et d'application des APCs.".to_string(),
            author: "Civium".to_string(),
            permissions: vec![],
            is_system: true,
            certification: CertificationLevel::Certified,
        }, true),
        (PluginManifest {
            id: "civium.messagerie".to_string(),
            name: "Messagerie".to_string(),
            version: "1.0.0".to_string(),
            description: "Fil de discussion chiffré et messages directs.".to_string(),
            author: "Civium".to_string(),
            permissions: vec![
                PluginPermission::ReadMessages,
                PluginPermission::WriteMessages,
                PluginPermission::ReadMembers,
            ],
            is_system: false,
            certification: CertificationLevel::Certified,
        }, true),
        (PluginManifest {
            id: "civium.annuaire".to_string(),
            name: "Annuaire".to_string(),
            version: "1.0.0".to_string(),
            description: "Annuaire de réseaux, membres et services.".to_string(),
            author: "Civium".to_string(),
            permissions: vec![
                PluginPermission::ReadDirectory,
                PluginPermission::WriteDirectory,
                PluginPermission::ReadMembers,
            ],
            is_system: false,
            certification: CertificationLevel::Certified,
        }, true),
        (PluginManifest {
            id: "civium.agenda".to_string(),
            name: "Agenda".to_string(),
            version: "1.0.0".to_string(),
            description: "Calendrier partagé — événements, lieux, récurrences.".to_string(),
            author: "Civium".to_string(),
            permissions: vec![
                PluginPermission::ReadAgenda,
                PluginPermission::WriteAgenda,
                PluginPermission::ReadMembers,
            ],
            is_system: false,
            certification: CertificationLevel::Certified,
        }, true),
        (PluginManifest {
            id: "civium.documents".to_string(),
            name: "Documents".to_string(),
            version: "1.0.0".to_string(),
            description: "Documents partagés et chiffrés au sein du réseau.".to_string(),
            author: "Civium".to_string(),
            permissions: vec![
                PluginPermission::ReadDocuments,
                PluginPermission::WriteDocuments,
                PluginPermission::ReadMembers,
            ],
            is_system: false,
            certification: CertificationLevel::Certified,
        }, true),
    ]
}
