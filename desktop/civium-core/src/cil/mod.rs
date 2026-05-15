use crate::plugin::{PluginPermission, PluginRecord, PluginState};

/// A data operation a plugin wants to perform through the CIL.
#[derive(Debug, Clone)]
pub enum CilAction {
    QueryMembers,
    QueryMessages,
    PostMessage,
    QueryProposals,
    CreateProposal,
    CastVote,
    QueryDirectory,
    PublishDirectory,
    QueryConnections,
    QueryAgenda,
    WriteAgenda,
}

impl CilAction {
    fn required_permission(&self) -> PluginPermission {
        match self {
            Self::QueryMembers      => PluginPermission::ReadMembers,
            Self::QueryMessages     => PluginPermission::ReadMessages,
            Self::PostMessage       => PluginPermission::WriteMessages,
            Self::QueryProposals    => PluginPermission::ReadGovernance,
            Self::CreateProposal    => PluginPermission::WriteGovernance,
            Self::CastVote          => PluginPermission::WriteGovernance,
            Self::QueryDirectory    => PluginPermission::ReadDirectory,
            Self::PublishDirectory  => PluginPermission::WriteDirectory,
            Self::QueryConnections  => PluginPermission::ReadConnections,
            Self::QueryAgenda       => PluginPermission::ReadAgenda,
            Self::WriteAgenda       => PluginPermission::WriteAgenda,
        }
    }
}

/// Check whether `plugin` is allowed to perform `action`.
/// Returns `Ok(())` if allowed, `Err(reason)` otherwise.
pub fn check_cil(plugin: &PluginRecord, action: &CilAction) -> Result<(), String> {
    if plugin.state != PluginState::Enabled {
        return Err(format!("plugin '{}' is not enabled", plugin.manifest.id));
    }
    let required = action.required_permission();
    if plugin.manifest.permissions.contains(&required) {
        Ok(())
    } else {
        Err(format!(
            "plugin '{}' n'a pas la permission '{required}' pour cette action",
            plugin.manifest.id
        ))
    }
}
