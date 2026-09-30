use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Confirming,
    Heating,
    Waiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivePhase {
    Confirming,
    Heating,
}

impl fmt::Display for ActivePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivePhase::Confirming => write!(f, "confirming"),
            ActivePhase::Heating => write!(f, "heating"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeatingCheckpoint {
    HeatingEnded,
    ConfirmFinishEnding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionRecordKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionRecord {
    pub endpoint: String,
    pub keys: PushSubscriptionRecordKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveEntry {
    pub id: String,
    pub name: String,
    pub session_token_hash: String,
    pub phase: ActivePhase,
    pub phase_started_at: u64,
    pub deadline: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_subscription: Option<PushSubscriptionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notified_checkpoints: Option<Vec<HeatingCheckpoint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitingEntry {
    pub id: String,
    pub name: String,
    pub session_token_hash: String,
    pub joined_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_subscription: Option<PushSubscriptionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatWaitlistEntry {
    pub id: String,
    pub token_hash: String,
    pub subscription: PushSubscriptionRecord,
    pub registered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueState {
    pub version: u64,
    pub active: Option<ActiveEntry>,
    pub waiting: Vec<WaitingEntry>,
    pub seat_waitlist: Vec<SeatWaitlistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifiedInput {
    pub id: String,
    pub session_token_hash: String,
}
