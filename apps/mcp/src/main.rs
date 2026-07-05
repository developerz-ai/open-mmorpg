//! Companion MCP — a player attaches their own AI agent to a **bounded
//! companion**, never to their main character.
//!
//! The safety property is a hard boundary: a companion can act within an
//! allow-list of low-stakes actions and can never issue main-character or
//! ownership-changing commands. We encode that as a type/allow-list here so the
//! dangerous action simply has no path (docs/architecture/07-mcp-companions.md).

/// Actions a companion agent may request. This is the *entire* surface — there
/// is deliberately no variant that controls the main character or moves items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionAction {
    /// Follow the owner at a distance.
    Follow,
    /// Report nearby points of interest.
    Scout,
    /// Emit an in-world emote.
    Emote,
}

/// Whether an action is permitted for a companion. Every current action is
/// bounded and safe; the function exists so new actions must be classified
/// explicitly rather than being allowed by default.
#[must_use]
pub fn companion_allowed(action: CompanionAction) -> bool {
    matches!(
        action,
        CompanionAction::Follow | CompanionAction::Scout | CompanionAction::Emote
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    tracing::info!("companion MCP server (stub): bounded actions only");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_actions_are_allowed() {
        assert!(companion_allowed(CompanionAction::Follow));
        assert!(companion_allowed(CompanionAction::Scout));
        assert!(companion_allowed(CompanionAction::Emote));
    }
}
