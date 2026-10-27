//! `robotctl stress` — a fixed lap of walks, provoked falls and sit/stands, run until the battery
//! is empty, timing every get-up. The design is
//! `docs/superpowers/specs/2026-10-27-stress-test-design.md`.

// Used by the driver, which comes next.
#[allow(dead_code)]
mod getup;
#[allow(dead_code)]
pub(crate) mod guard;
#[allow(dead_code)]
mod lap;
