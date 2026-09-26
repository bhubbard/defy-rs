//! Benchmark comparing `defy-rs` (Rust) vs openfw-game/defy (C# / FiveM Lua police pursuit & NPC crime engine).

use defy_rs::npc::ThreatStimulus;
use defy_rs::{
    BystanderReaction, CrimeType, DispatchUnitType, NpcContext, NpcEntity, NpcPersonality,
    PoliceUnit, WantedLevelSystem, WantedStatus, WitnessSystem,
};
use glam::Vec3;
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("    defy-rs (Rust) vs openfw-game/defy (C# / FiveM Lua)    ");
    println!("============================================================");

    // 1. Multi-Unit Police Pursuit & Wanted Level Update (50 Police Units)
    println!("\n--- 1. Multi-Unit Police Pursuit & AI State Machine (50 Units) ---");
    {
        let mut system = WantedLevelSystem::default();
        system.set_minimum_stars(4, Vec3::ZERO);

        let mut units: Vec<PoliceUnit> = (0..50)
            .map(|i| {
                let u_type = match i % 4 {
                    0 => DispatchUnitType::StandardCruiser,
                    1 => DispatchUnitType::UnmarkedCruiser,
                    2 => DispatchUnitType::SwatEnforcerVan,
                    _ => DispatchUnitType::PoliceHelicopter,
                };
                PoliceUnit::new(i as u64, u_type, Vec3::new(i as f32 * 5.0, 0.0, i as f32 * 5.0))
            })
            .collect();

        let iterations = 100_000;
        let dt = 0.016;
        let start = Instant::now();
        let mut total_active_pursuits = 0;

        for i in 0..iterations {
            let t = (i as f32) * 0.01;
            let suspect_pos = Vec3::new(t.sin() * 200.0, 0.0, t.cos() * 200.0);
            let suspect_speed = 35.0 + (i % 30) as f32;

            // Toggle line-of-sight dynamically
            for (idx, u) in units.iter_mut().enumerate() {
                u.has_line_of_sight = (i + idx) % 5 != 0;
            }

            system.update(dt, suspect_pos, suspect_speed, &mut units);
            if system.status == WantedStatus::ActivePursuit {
                total_active_pursuits += 1;
            }
        }

        std::hint::black_box(total_active_pursuits);
        let elapsed = start.elapsed();
        let ns_per_frame = elapsed.as_nanos() as f64 / iterations as f64;
        let frames_per_sec = iterations as f64 / elapsed.as_secs_f64();
        let ns_per_unit = ns_per_frame / 50.0;

        println!(
            "Pursuit Frames (50 Units): {} | Time: {:.2?} | Latency: {:.2} µs/frame ({:.2} ns/unit) | {:>10.0} frames/s",
            iterations, elapsed, ns_per_frame / 1000.0, ns_per_unit, frames_per_sec
        );
    }

    // 2. NPC Bystander Sensory Perception & Reaction Triage (200 NPCs)
    println!("\n--- 2. NPC Bystander Reaction & Fear Triage (200 NPCs) ---");
    {
        let mut npcs: Vec<NpcEntity> = (0..200)
            .map(|i| {
                let personality = match i % 4 {
                    0 => NpcPersonality::Coward,
                    1 => NpcPersonality::Aggressive,
                    2 => NpcPersonality::ArmedVigilante,
                    _ => NpcPersonality::AverageCitizen,
                };
                NpcEntity::new(
                    i as u64,
                    Vec3::new((i % 20) as f32 * 10.0, 0.0, (i / 20) as f32 * 10.0),
                    personality,
                )
            })
            .collect();

        let iterations = 50_000;
        let start = Instant::now();
        let mut fleeing_count = 0;

        for i in 0..iterations {
            let threat_pos = Vec3::new((i % 100) as f32, 0.0, (i % 50) as f32);
            let ctx = NpcContext {
                threats: vec![ThreatStimulus {
                    source_id: 999,
                    position: threat_pos,
                    is_gunfire: true,
                    is_aimed_at_me: false,
                    is_melee_attack: false,
                }],
                delta_time: 0.016,
            };

            for npc in &mut npcs {
                npc.update(&ctx);
                if matches!(npc.reaction, BystanderReaction::Flee { .. } | BystanderReaction::Cower) {
                    fleeing_count += 1;
                }
            }
        }

        std::hint::black_box(fleeing_count);
        let elapsed = start.elapsed();
        let total_evals = iterations * 200;
        let ns_per_eval = elapsed.as_nanos() as f64 / total_evals as f64;
        let evals_per_sec = total_evals as f64 / elapsed.as_secs_f64();

        println!(
            "Bystander Evals: {} | Time: {:.2?} | Latency: {:.2} ns/npc | {:>10.0} evals/s | Panicking: {}",
            total_evals, elapsed, ns_per_eval, evals_per_sec, fleeing_count
        );
    }

    // 3. Crime Event Detection & Witness 911 Call System
    println!("\n--- 3. Crime Witness Sensory Processing & 911 Call Dispatch ---");
    {
        let iterations = 5_000_000;
        let start = Instant::now();
        let mut detected_crimes = 0;

        for i in 0..iterations {
            let observer_pos = Vec3::ZERO;
            let observer_forward = Vec3::Z;
            let target_pos = Vec3::new((i % 100) as f32 - 50.0, 0.0, (i % 80) as f32);

            let can_see = WitnessSystem::can_detect(
                observer_pos,
                observer_forward,
                40.0,
                1.57, // 90 deg FOV
                target_pos,
                CrimeType::ArmedRobbery,
                true,
            );

            if can_see {
                detected_crimes += 1;
            }
        }

        std::hint::black_box(detected_crimes);
        let elapsed = start.elapsed();
        let ns_per_eval = elapsed.as_nanos() as f64 / iterations as f64;
        let evals_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Sensory Detections: {} | Time: {:.2?} | Latency: {:.2} ns/eval | {:>10.0} evals/s | Detected: {}",
            iterations, elapsed, ns_per_eval, evals_per_sec, detected_crimes
        );
    }

    // 4. Star Progression & Dynamic Crime Score Scaling
    println!("\n--- 4. Star Progression & Dynamic Score Scaling ---");
    {
        let mut system = WantedLevelSystem::default();
        let iterations = 10_000_000;
        let start = Instant::now();
        let mut stars_sum = 0u32;

        for i in 0..iterations {
            let score = 50 + (i % 2000) as u32;
            system.add_crime_score(score, Vec3::ZERO);
            stars_sum += system.stars as u32;

            if i % 100 == 0 {
                system.clear();
            }
        }

        std::hint::black_box(stars_sum);
        let elapsed = start.elapsed();
        let ns_per_eval = elapsed.as_nanos() as f64 / iterations as f64;
        let evals_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Score Updates: {} | Time: {:.2?} | Latency: {:.2} ns/eval | {:>10.0} evals/s",
            iterations, elapsed, ns_per_eval, evals_per_sec
        );
    }

    println!("\n============================================================");
    println!("                      Benchmark Complete                    ");
    println!("============================================================");
}
