//! Audit, replay packages, and verification.
//!
//! Combat execution remains in `rulebench-combat`; this crate owns replay
//! specifications and immutable evidence comparison.

#![forbid(unsafe_code)]

mod automatic_run;

pub use automatic_run::{
    verify_automatic_run_replay, CombatSessionAutomaticRunReplayDecisionKind,
    CombatSessionAutomaticRunReplayReadout, CombatSessionAutomaticRunReplaySpec,
};
