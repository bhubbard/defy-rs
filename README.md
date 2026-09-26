# defy-rs

[![GitHub Pages](https://img.shields.io/badge/docs-GitHub%20Pages-blue?style=flat-square&logo=github)](https://code.brandonhubbard.com/defy-rs/)
[![Tests](https://img.shields.io/badge/tests-16%20passed-success?style=flat-square&logo=rust)](https://github.com/bhubbard/defy-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square)](LICENSE-MIT)

Pure Rust port of police wanted-level, pursuit dispatch, and NPC crime interaction systems translated from [openfw-game/defy](https://github.com/openfw-game/defy) (open-source GTA clone). Designed for Bevy ECS and modern game architectures.

## Features

### 1. Police Wanted-Level System (`defy_rs::wanted`)
- **1 to 6 Stars Progression**: Star rating determined dynamically by accumulated crime severity score or direct threshold triggers.
- **Dispatch Compositions**: Star-dependent police unit composition, spawn intervals, active unit caps, and roadblock/spike probabilities:
  - **1 Star**: Routine patrol cruisers, peaceful surrender / arrest attempts.
  - **2 Stars**: Aggressive cruisers, tactical PIT maneuvers and flanking.
  - **3 Stars**: Enforcer vans, roadblocks with spike strips, Police Maverick helicopter air surveillance.
  - **4 Stars**: SWAT tactical units, automatic weapons, heavy barricades.
  - **5 Stars**: FIB tactical SUVs, intense helicopter snipers and searchlights.
  - **6 Stars**: Military martial law, Rhino tanks, Barracks trucks, and shoot-on-sight lethal lockdown.
- **Pursuit AI State Machine**:
  - `Patrol`: Ambient patrol routes.
  - `Investigate`: Searching last known suspect coordinates.
  - `Pursue`: Active visual chase.
  - `Arrest`: Cornered suspect Busted countdown for non-violent crimes.
  - `DeadlyForce`: Shoot-on-sight lethal engagement at high star ratings or violent resistance.
- **Line-of-Sight & Decay**:
  - Out of visual contact triggers `SearchingFlashing` status and search radius bubble.
  - Cooldown timer decays wanted level stars progressively down to clean status.
  - Re-detection immediately refreshes pursuit and updates last known position.

### 2. Crime Reporting & Witness System (`defy_rs::crime`)
- **Crime Classification**: Infractions categorized by severity (`Minor`, `Moderate`, `Felony`, `Capital`) with calibrated score points and sound loudness radius.
- **Sensory Detection**: Multi-modal detection supporting visual sight cones (FOV angle + distance) and omnidirectional acoustic hearing (gunfire, car alarms, explosions).
- **911 Call Pipelines**: Witnesses experience reaction shock latency before dialing 911 over a timed duration.
- **Silenceable Witnesses**: If a player neutralizes, knocks out, intimidates, or eliminates a witness before the 911 call completes, the report is canceled.

### 3. NPC Interaction States (`defy_rs::npc`)
- **Bystander Reactions**: `Wander`, `Flee`, `HandsUp`, `FightBack`, `Cower`.
- **Personality Archetypes**:
  - `Coward`: Flees or cowers in terror at loud sounds and gunshots.
  - `AverageCitizen`: Surrenders at gunpoint, cowers at close gunfire, flees if safe.
  - `Aggressive`: Retaliates against melee attacks and carjackings.
  - `ArmedVigilante`: Conceals a weapon and returns fire against violent threats.
- **Agitation Decays**: Panic timers naturally wind down when danger subsides, returning NPCs to ambient strolls.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
defy-rs = { git = "https://github.com/bhubbard/defy-rs" }
glam = "0.29"
```

## Quick Example

```rust
use glam::Vec3;
use defy_rs::{
    WantedLevelSystem, WantedStatus, PoliceUnit, DispatchUnitType,
    WitnessSystem, CrimeEvent, CrimeType,
    NpcEntity, NpcPersonality, NpcContext, ThreatStimulus, BystanderReaction,
};

fn main() {
    // 1. Wanted System
    let mut wanted = WantedLevelSystem::default();
    wanted.add_crime_score(150, Vec3::ZERO); // 1 Star
    assert_eq!(wanted.stars, 1);

    // 2. Witness & 911 Calls
    let mut witnesses = WitnessSystem::new();
    witnesses.register_witness(
        1,
        Vec3::new(5.0, 0.0, 5.0),
        CrimeEvent {
            crime_type: CrimeType::VehicleTheft,
            perpetrator_id: 100,
            location: Vec3::ZERO,
            timestamp_sec: 0.0,
            is_direct_police_view: false,
        },
        3.0,
    );

    // Player intimidates witness before call finishes:
    assert!(witnesses.silence_witness(1));

    // 3. NPC Reactions
    let mut bystander = NpcEntity::new(10, Vec3::ZERO, NpcPersonality::AverageCitizen);
    let ctx = NpcContext {
        threats: vec![ThreatStimulus {
            source_id: 100,
            position: Vec3::new(0.0, 0.0, 3.0),
            is_gunfire: false,
            is_aimed_at_me: true,
            is_melee_attack: false,
        }],
        delta_time: 0.016,
    };
    bystander.update(&ctx);
    assert_eq!(bystander.reaction, BystanderReaction::HandsUp);
}
```

## Running Tests

```bash
cargo test
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
