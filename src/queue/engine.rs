use crate::queue::{
    error::QueueError::{self, Forbidden, NotFound, WrongPhase},
    types::{ActiveEntry, ActivePhase, IdentifiedInput, QueueState},
};

const CONFIRM_WINDOW_MS: u64 = 60_000;

const HEATING_NOMINAL_MS: u64 = 300_000;

const HEATING_URGENCY_MS: u64 = 30_000;

const HEATING_WINDOW_MS: u64 = HEATING_NOMINAL_MS + HEATING_URGENCY_MS;

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

pub fn apply_leave(state: &QueueState, input: &IdentifiedInput) -> Result<QueueState, QueueError> {
    if state
        .active
        .as_ref()
        .is_some_and(|active| active.id == input.id)
    {
        return Err(WrongPhase(String::from(
            "Cannot leave an active turn - only finishing it is supported",
        )));
    }

    let Some(target) = state.waiting.iter().find(|waiting| waiting.id == input.id) else {
        return Err(NotFound(input.id.clone()));
    };

    if target.session_token_hash != input.session_token_hash {
        return Err(Forbidden(input.id.clone()));
    }

    let mut next_state = state.clone();

    next_state.waiting.retain(|waiting| waiting.id != input.id);

    Ok(next_state)
}

pub fn apply_confirm_turn(
    state: &QueueState,
    input: &IdentifiedInput,
    now: u64,
) -> Result<QueueState, QueueError> {
    let Some(active) = &state.active else {
        return Err(NotFound(input.id.clone()));
    };

    if active.id != input.id {
        return Err(NotFound(input.id.clone()));
    }

    if active.phase != ActivePhase::Confirming {
        let phase = active.phase;
        let message = format!(
            "Cannot confirm turn: active entry is in \"{phase}\" phase, not \"confirming\""
        );

        return Err(WrongPhase(message));
    }

    if active.session_token_hash != input.session_token_hash {
        return Err(Forbidden(input.id.clone()));
    }

    let mut next_state = state.clone();

    if let Some(active_mut) = next_state.active.as_mut() {
        active_mut.phase = ActivePhase::Heating;
        active_mut.phase_started_at = now;
        active_mut.deadline = now + HEATING_WINDOW_MS;
    }

    Ok(next_state)
}
