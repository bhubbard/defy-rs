use glam::Vec3;
use defy_rs::{
    BystanderReaction, NpcContext, NpcEntity, NpcPersonality,
};
use defy_rs::npc::ThreatStimulus;

#[test]
fn test_coward_cowers_on_gunfire() {
    let mut npc = NpcEntity::new(1, Vec3::ZERO, NpcPersonality::Coward);
    assert_eq!(npc.reaction, BystanderReaction::Wander);

    let ctx = NpcContext {
        threats: vec![ThreatStimulus {
            source_id: 10,
            position: Vec3::new(5.0, 0.0, 5.0),
            is_gunfire: true,
            is_aimed_at_me: false,
            is_melee_attack: false,
        }],
        delta_time: 0.016,
    };

    npc.update(&ctx);
    assert_eq!(npc.reaction, BystanderReaction::Cower);
    assert!(npc.panic_timer > 0.0);
}

#[test]
fn test_average_citizen_hands_up_at_gunpoint() {
    let mut npc = NpcEntity::new(2, Vec3::ZERO, NpcPersonality::AverageCitizen);

    let ctx = NpcContext {
        threats: vec![ThreatStimulus {
            source_id: 10,
            position: Vec3::new(0.0, 0.0, 3.0), // Close distance 3m
            is_gunfire: false,
            is_aimed_at_me: true,
            is_melee_attack: false,
        }],
        delta_time: 0.016,
    };

    npc.update(&ctx);
    assert_eq!(npc.reaction, BystanderReaction::HandsUp);
}

#[test]
fn test_aggressive_fights_back_on_melee() {
    let mut npc = NpcEntity::new(3, Vec3::ZERO, NpcPersonality::Aggressive);

    let ctx = NpcContext {
        threats: vec![ThreatStimulus {
            source_id: 77,
            position: Vec3::new(1.0, 0.0, 0.0),
            is_gunfire: false,
            is_aimed_at_me: false,
            is_melee_attack: true,
        }],
        delta_time: 0.016,
    };

    npc.update(&ctx);
    assert_eq!(
        npc.reaction,
        BystanderReaction::FightBack {
            target_id: 77,
            is_armed: false,
        }
    );
}

#[test]
fn test_armed_vigilante_draws_weapon_on_gunfire() {
    let mut npc = NpcEntity::new(4, Vec3::ZERO, NpcPersonality::ArmedVigilante);

    let ctx = NpcContext {
        threats: vec![ThreatStimulus {
            source_id: 88,
            position: Vec3::new(10.0, 0.0, 10.0),
            is_gunfire: true,
            is_aimed_at_me: false,
            is_melee_attack: false,
        }],
        delta_time: 0.016,
    };

    npc.update(&ctx);
    assert_eq!(
        npc.reaction,
        BystanderReaction::FightBack {
            target_id: 88,
            is_armed: true,
        }
    );
}

#[test]
fn test_panic_timer_calms_down() {
    let mut npc = NpcEntity::new(5, Vec3::ZERO, NpcPersonality::Coward);

    let ctx_threat = NpcContext {
        threats: vec![ThreatStimulus {
            source_id: 10,
            position: Vec3::new(20.0, 0.0, 20.0),
            is_gunfire: true,
            is_aimed_at_me: false,
            is_melee_attack: false,
        }],
        delta_time: 0.016,
    };
    npc.update(&ctx_threat);
    assert!(matches!(npc.reaction, BystanderReaction::Flee { .. }));

    // Advance empty context past panic timer
    let ctx_calm = NpcContext {
        threats: vec![],
        delta_time: 15.0,
    };
    npc.update(&ctx_calm);
    assert_eq!(npc.reaction, BystanderReaction::Wander);
}
