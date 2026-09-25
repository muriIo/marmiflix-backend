use crate::queue::types::{ActiveEntry, ActivePhase, QueueState};

const CONFIRM_WINDOW_MS: u64 = 60_000;

fn normalized_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn promote_next_to_active(state: &QueueState, now: u64) -> QueueState {
    let mut next_state = state.clone();

    if next_state.waiting.is_empty() {
        next_state.active = None;

        return next_state;
    }

    let next = next_state.waiting.remove(0);

    next_state.active = Some(ActiveEntry {
        id: next.id,
        name: next.name,
        session_token_hash: next.session_token_hash,
        phase: ActivePhase::Confirming,
        phase_started_at: now,
        deadline: now + CONFIRM_WINDOW_MS,
        push_subscription: next.push_subscription,
        notified_checkpoints: None,
    });

    next_state
}

pub fn reap_expired(state: &QueueState, now: u64) -> QueueState {
    if let Some(active) = &state.active {
        if now > active.deadline {
            return promote_next_to_active(state, now);
        }
    }

    state.clone()
}
