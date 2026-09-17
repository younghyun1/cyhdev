//! Authoritative account checks and bounded Minecraft operations.

use super::super::domain::command::Command;
use super::transport::ManagementTransport;
use crate::features::accounts::{
    authorization_error::AuthorizationError, service::account_service::AccountService,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::Instant};
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum ManagementError {
    #[error(transparent)]
    Authority(#[from] AuthorizationError),
    #[error("Invalid Minecraft control input")]
    Invalid,
    #[error("Minecraft controls are not configured")]
    Disabled,
    #[error("A Minecraft operation is in progress or cooling down")]
    Busy,
    #[error("Minecraft did not acknowledge the request; check its state before retrying")]
    Uncertain,
}

#[derive(Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
}

pub struct Status {
    pub players: Vec<Player>,
    pub whitelist: Vec<Player>,
    pub whitelist_enabled: bool,
}

pub struct ManagementService {
    accounts: Arc<AccountService>,
    transport: Option<ManagementTransport>,
    // One bounded slot, with no queued commands surviving a browser timeout.
    gate: Mutex<Instant>,
}

impl ManagementService {
    pub fn from_environment(accounts: Arc<AccountService>) -> anyhow::Result<Self> {
        Ok(Self {
            accounts,
            transport: ManagementTransport::from_environment()?,
            gate: Mutex::new(Instant::now()),
        })
    }

    pub async fn status(&self, actor: Uuid) -> Result<Status, ManagementError> {
        let _authority = self
            .accounts
            .acquire_current_younghyun_authority(actor)
            .await?;
        let _gate = self.gate.try_lock().map_err(|_| ManagementError::Busy)?;
        let transport = self.transport.as_ref().ok_or(ManagementError::Disabled)?;
        let result = tokio::time::timeout(Duration::from_secs(10), async {
            let players =
                serde_json::from_value(transport.call("minecraft:players", json!([])).await?)?;
            let whitelist =
                serde_json::from_value(transport.call("minecraft:allowlist", json!([])).await?)?;
            let whitelist_enabled = serde_json::from_value(
                transport
                    .call("minecraft:serversettings/use_allowlist", json!([]))
                    .await?,
            )?;
            Ok::<_, anyhow::Error>(Status {
                players,
                whitelist,
                whitelist_enabled,
            })
        })
        .await;
        match result {
            Ok(Ok(status)) => Ok(status),
            // Do not log transport errors: handshake errors can contain sensitive headers.
            _ => {
                tracing::warn!(%actor, "Minecraft status unavailable");
                Err(ManagementError::Uncertain)
            }
        }
    }

    pub async fn execute(&self, actor: Uuid, command: Command) -> Result<(), ManagementError> {
        if !command.valid() {
            return Err(ManagementError::Invalid);
        }
        let _authority = self
            .accounts
            .acquire_current_younghyun_authority(actor)
            .await?;
        let mut gate = self.gate.try_lock().map_err(|_| ManagementError::Busy)?;
        if Instant::now() < *gate {
            return Err(ManagementError::Busy);
        }
        let transport = self.transport.as_ref().ok_or(ManagementError::Disabled)?;
        let cooldown = match command {
            Command::Restart => 60,
            Command::Save => 10,
            _ => 2,
        };
        *gate = Instant::now() + Duration::from_secs(cooldown);
        let expected_bool = match command {
            Command::WhitelistEnable(enabled) => enabled,
            _ => true,
        };
        let (method, params) = rpc_command(command);
        tracing::info!(%actor, method, "Minecraft control requested");
        let result =
            tokio::time::timeout(Duration::from_secs(15), transport.call(method, params)).await;
        let accepted = matches!(result, Ok(Ok(ref value)) if value == &json!(expected_bool) || value.is_array());
        tracing::info!(%actor, method, acknowledged = accepted, "Minecraft control result");
        if accepted {
            Ok(())
        } else {
            Err(ManagementError::Uncertain)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_and_literal_messages() {
        assert!(!Command::Message("\nstop".into()).valid());
        assert!(!Command::Message(" ".into()).valid());
        assert!(Command::Message("한".repeat(512)).valid());
        assert!(!Command::Message("한".repeat(513)).valid());
        for name in ["", "a b", "@a", "x\nstop", "12345678901234567"] {
            assert!(!Command::Kick(name.into()).valid());
        }
        assert!(Command::WhitelistAdd("Player_123".into()).valid());
        let (method, params) = rpc_command(Command::Message("/stop \"hello\"".into()));
        assert_eq!(method, "minecraft:server/system_message");
        assert_eq!(params[0]["message"]["literal"], "/stop \"hello\"");
        assert!(params[0].get("receivingPlayers").is_none());
    }

    #[test]
    fn controls_use_fixed_rpc_methods() {
        assert_eq!(
            rpc_command(Command::Restart),
            ("minecraft:server/stop", json!([]))
        );
        assert_eq!(
            rpc_command(Command::Save),
            ("minecraft:server/save", json!([true]))
        );
        assert_eq!(
            rpc_command(Command::WhitelistEnable(false)),
            ("minecraft:serversettings/use_allowlist/set", json!([false]))
        );
        assert_eq!(
            rpc_command(Command::Kick("Alex".into())).1,
            json!([[{"player":{"name":"Alex"}}]])
        );
    }
}

/// Typed commands prevent arbitrary console execution and message interpretation.
fn rpc_command(command: Command) -> (&'static str, Value) {
    match command {
        Command::Message(message) => (
            "minecraft:server/system_message",
            json!([{"message":{"literal":message},"overlay":false}]),
        ),
        Command::WhitelistAdd(name) => ("minecraft:allowlist/add", json!([[{"name":name}]])),
        Command::WhitelistRemove(name) => ("minecraft:allowlist/remove", json!([[{"name":name}]])),
        Command::WhitelistEnable(enabled) => (
            "minecraft:serversettings/use_allowlist/set",
            json!([enabled]),
        ),
        Command::Kick(name) => (
            "minecraft:players/kick",
            json!([[{"player":{"name":name}}]]),
        ),
        Command::Save => ("minecraft:server/save", json!([true])),
        Command::Restart => ("minecraft:server/stop", json!([])),
    }
}
