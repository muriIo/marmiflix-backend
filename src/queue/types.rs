use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Confirming,
    Heating,
    Waiting,
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
    pub phase: Phase,
    pub phase_started_at: u64,
    pub deadline: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_subscription: Option<PushSubscriptionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notified_checkpoints: Option<HeatingCheckpoint>,
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
    version: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<ActiveEntry>,
    waiting: Vec<WaitingEntry>,
    seat_waitlist: Vec<SeatWaitlistEntry>,
}
