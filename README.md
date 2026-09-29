# marmiflix-backend

A Rust port of the marmiflix queue backend: the service that manages the shared
microwave queue (join, confirm your turn, heat, finish), the seat waitlist, and
the web-push notifications.

The original backend is part of the Next.js app. It has a pure state machine
(`lib/queue/engine.ts`), a Redis-backed compare-and-swap store, eight HTTP
routes, and a web-push dispatcher. This repo re-implements it by hand, one
slice at a time, as a way to learn Rust. The goal is a service that matches
the existing JSON contracts closely enough that the frontend can switch to it
without changes.

## Stack

- **HTTP:** [axum](https://github.com/tokio-rs/axum) 0.8 on tokio
- **Middleware:** tower-http (request id, tracing, CORS, timeout, body limit, gzip)
- **Config:** dotenvy + envy (loads `.env` into a typed `Config` struct)
- **Serialization:** serde / serde_json
- **Errors:** thiserror
- **Logging:** tracing-subscriber with env-filter
- **State store (planned):** Redis

## Project layout

```
src/
├── main.rs          # Config loading, AppState, router, middleware stack, server
└── queue/
    ├── mod.rs
    ├── types.rs     # QueueState, ActiveEntry, WaitingEntry, SeatWaitlistEntry,
    │                # Phase / ActivePhase, HeatingCheckpoint, push subscription records
    ├── error.rs     # QueueError (DuplicateName, NotFound, Forbidden, WrongPhase,
    │                # QueueBusy, Validation, QueueFull)
    └── engine.rs    # Pure state-machine functions (no I/O)
```

The types use `#[serde(rename_all = "camelCase")]`, so the serialized JSON
matches the shape of the TypeScript `QueueState`.

## Running locally

```sh
cp .env.example .env     # then fill in the values
cargo run                # listens on 0.0.0.0:3000
```

Endpoints available today:

| Method | Path      | Response                 |
|--------|-----------|--------------------------|
| GET    | `/`       | `hello world`            |
| GET    | `/health` | `{"status_code": 200}`   |

### Configuration

`Config::from_env` loads `.env` (overriding existing env vars) and deserializes
these variables. Every one must be present, but it may be empty:

- `UPSTASH_REDIS_REST_URL`, `UPSTASH_REDIS_REST_TOKEN`
- `VAPID_PUBLIC_KEY`, `VAPID_PRIVATE_KEY`, `VAPID_SUBJECT`
- `QUEUE_CONFIRM_WINDOW_SECONDS`, `QUEUE_HEATING_NOMINAL_SECONDS`,
  `QUEUE_HEATING_URGENCY_SECONDS`, `QUEUE_PER_PERSON_WAIT_SECONDS`
- `SENTRY_DSN`

See `.env.example` for what each one means. The `NEXT_PUBLIC_*` entries there
are for the frontend and this service ignores them.

### Local Redis

```sh
docker compose -f docker-compose.test.yml up -d
```

This starts `redis:7-alpine` on `6379` and a
[serverless-redis-http](https://github.com/hiett/serverless-redis-http) proxy on
`8079` (token `local-dev-token`), which mimics the Upstash REST API.

## Status

**Done**

- [x] axum service with `/health`, request-id, tracing, CORS, timeout, body
      limit and compression layers
- [x] `.env` loading into a typed `Config`
- [x] Redis + REST proxy in `docker-compose.test.yml`
- [x] Queue domain types with serde, matching the TS JSON shape
- [x] `QueueError` enum with the 7 domain error variants
- [x] Engine: `promote_next_to_active`, `reap_expired`

**Known gaps in the current code**

- `CONFIRM_WINDOW_MS` is hardcoded in `engine.rs`. It should come from
  `QUEUE_CONFIRM_WINDOW_SECONDS`.
- The `QUEUE_*` timing fields in `Config` are `String`. They should be parsed
  into numbers, with the defaults from `.env.example` when a value is missing
  or invalid.
- `CorsLayer::permissive()` is a placeholder until a real origin policy exists.
- There are no tests yet.

## Roadmap

These are the next steps, in order. Each phase ports one real slice of the
TypeScript backend and leaves something that runs end-to-end.

### 1. Finish the pure queue engine
Source: `lib/queue/engine.ts`, `lib/queue/types.ts`

- [ ] `apply_join` (use `normalized_name` for duplicate detection → `DuplicateName`, `QueueFull`)
- [ ] `apply_leave`
- [ ] `apply_confirm_turn`
- [ ] `apply_finish_heating`
- [ ] `apply_heating_checkpoints`
- [ ] `apply_prune_subscriptions`
- [ ] Port every case from the TS engine test suite as `#[test]` functions

### 2. In-memory store and concurrency
Source: `lib/queue/store.ts` (shape only)

- [ ] Shared `Arc<RwLock<QueueState>>` in `AppState`
- [ ] `with_queue_mutation(f)`: read → reap expired → apply checkpoints → mutate → write
- [ ] Return `(result, notification_jobs)` like the TS version
- [ ] Concurrency test: N parallel joins, no lost updates

### 3. HTTP layer
Source: `app/api/queue/**/route.ts`

- [ ] `GET /api/queue?id=`
- [ ] `POST /api/queue/join`, `/leave`, `/confirm-turn`, `/finish`, `/push-subscribe`
- [ ] `POST /api/queue/waitlist/join`, `/waitlist/leave`
- [ ] Map `QueueError` to the same status codes as the TS routes (400 / 403 / 404 / 409 / 429 / 503) via `IntoResponse`
- [ ] Diff JSON responses against `app/api/queue/__tests__`

### 4. Session and auth
Source: `lib/queue/session.ts`, `lib/queue/route-helpers.ts` (crates: `rand`, `sha2`, `subtle`)

- [ ] `generate_session_token()`: CSPRNG, base64url
- [ ] `hash_token()` with SHA-256
- [ ] `verify_token()` with `subtle::ConstantTimeEq`
- [ ] `authorize_entry`: the 404-then-403 gate for leave, confirm-turn and finish

### 5. Rate limiting
Source: `lib/queue/rate-limit.ts`

- [ ] Per-IP join limit (INCR + EXPIRE window semantics)
- [ ] In-memory version first (`DashMap<String, (u32, Instant)>`)
- [ ] Swap to Redis after phase 7 without changing the signature

### 6. View and wait-estimate math
Source: `lib/queue/view.ts`

- [ ] `remaining_active_ms` (confirming vs heating)
- [ ] `build_view`: waiting viewer, active viewer, anonymous
- [ ] Unit tests with fixed `(state, now)` pairs

### 7. Redis and compare-and-swap
Source: `lib/queue/store.ts`, `lib/queue/redis-client.ts` (crate: `redis` with `tokio-comp`)

- [ ] Port `CAS_SCRIPT` via `redis::Script` on key `queue:state`
- [ ] Retry loop: up to 5 attempts, 5–25 ms jittered backoff, `QueueBusy` on exhaustion
- [ ] Use native RESP instead of the Upstash REST wrapper
- [ ] Run `loadtest/queue-load-test.js` against it

### 8. Web push notifications
Source: `lib/notifications/*` (crate: `web-push`)

- [ ] 4 strategies as an enum: turn-ready, heating-ended, confirm-finish-ending, seat-opened
- [ ] `dispatch_notification_job` with `futures::future::join_all`
- [ ] Prune subscriptions on 404/410
- [ ] Dispatch after responding with `tokio::spawn`

### 9. Observability and hardening

- [ ] Per-request tracing spans (request id, route, status)
- [ ] `sentry` crate for exception capture
- [ ] CORS scoped to the Next.js origin
- [ ] Graceful shutdown on SIGTERM

### 10. Test parity

- [ ] All engine unit tests ported
- [ ] Redis integration tests via `testcontainers`
- [ ] Replay the vitest integration fixtures against this service
- [ ] Load-test comparison against the Node baseline

### 11. Deploy and cut over

- [ ] Multi-stage Dockerfile (release build, slim runtime)
- [ ] Deploy to a host that runs a long-lived binary (Fly.io, Railway, a VPS)
- [ ] Run both backends in parallel behind a subdomain or proxy
- [ ] Verify parity, then remove `app/api/queue/**` from the Next.js app

### Stretch goals

- [ ] SSE or WebSocket stream instead of polling `GET /api/queue`
- [ ] Redis pub/sub over a long-lived connection
- [ ] Actor-model mutation loop (one task owns `QueueState`) as an alternative to CAS retry
- [ ] Node vs Rust benchmark write-up
