//! # defy-rs
//!
//! Pure Rust port of police wanted-level, pursuit dispatch, and NPC crime interaction systems
//! translated from `openfw-game/defy` (open-source GTA-style sandbox).
//!
//! - **`wanted`**: 1 to 6 stars progression, crime severity thresholds, dynamic unit dispatch composition, pursuit AI states, and line-of-sight cooldown decay.
//! - **`crime`**: Crime events, sensory witness detection (visual cone + audio hearing), 911 call countdown timers, and silenceable witnesses.
//! - **`npc`**: Bystander reactions (Wander, Flee, HandsUp, FightBack, Cower) governed by personality traits and sensory stimuli.

pub mod crime;
pub mod npc;
pub mod wanted;

pub use crime::{
    CrimeEvent, CrimeSeverity, CrimeType, SensoryStimulus, StimulusKind, WitnessCallState,
    WitnessSystem,
};
pub use npc::{
    BystanderReaction, NpcContext, NpcEntity, NpcPersonality, NpcSenses, ThreatStimulus,
};
pub use wanted::{
    DispatchComposition, DispatchUnitType, PoliceUnit, PursuitAiState, PursuitConfig,
    WantedLevelSystem, WantedStatus,
};
