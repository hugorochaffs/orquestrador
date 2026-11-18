//! STUB (agente A): wizard dialoguer.
use crate::types::SessionConfig;

/// args = argv sem o nome do programa: `<dir> [--mock-planner]`.
pub fn parse_and_wizard(_args: Vec<String>) -> Result<SessionConfig, String> {
    Err("wizard não implementado".into())
}
