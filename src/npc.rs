use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Bystander interaction and reaction state in open-world GTA clone.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum BystanderReaction {
    #[default]
    Wander,
    Flee {
        threat_source: Vec3,
    },
    HandsUp,
    FightBack {
        target_id: u64,
        is_armed: bool,
    },
    Cower,
}

/// Personality archetype determining how an NPC reacts to danger and aggression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NpcPersonality {
    Coward,
    #[default]
    AverageCitizen,
    Aggressive,
    ArmedVigilante,
}

/// Sensory configuration of an NPC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcSenses {
    pub sight_range: f32,
    pub fov_angle_rad: f32,
    pub hearing_range: f32,
}

impl Default for NpcSenses {
    fn default() -> Self {
        Self {
            sight_range: 35.0,
            fov_angle_rad: 110.0_f32.to_radians(),
            hearing_range: 40.0,
        }
    }
}

/// Environmental stimulus / threat perceived by an NPC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatStimulus {
    pub source_id: u64,
    pub position: Vec3,
    pub is_gunfire: bool,
    pub is_aimed_at_me: bool,
    pub is_melee_attack: bool,
}

/// Simulation context passed during NPC update tick.
#[derive(Debug, Clone, Default)]
pub struct NpcContext {
    pub threats: Vec<ThreatStimulus>,
    pub delta_time: f32,
}

/// Individual civilian NPC representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcEntity {
    pub id: u64,
    pub position: Vec3,
    pub forward: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub personality: NpcPersonality,
    pub senses: NpcSenses,
    pub reaction: BystanderReaction,
    pub panic_timer: f32,
}

impl NpcEntity {
    pub fn new(id: u64, position: Vec3, personality: NpcPersonality) -> Self {
        Self {
            id,
            position,
            forward: Vec3::Z,
            velocity: Vec3::ZERO,
            health: 100.0,
            max_health: 100.0,
            personality,
            senses: NpcSenses::default(),
            reaction: BystanderReaction::Wander,
            panic_timer: 0.0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Evaluates threats and sensory inputs to transition bystander reaction state.
    pub fn update(&mut self, ctx: &NpcContext) {
        if !self.is_alive() {
            return;
        }

        if self.panic_timer > 0.0 {
            self.panic_timer = (self.panic_timer - ctx.delta_time).max(0.0);
            if self.panic_timer == 0.0 && !matches!(self.reaction, BystanderReaction::Wander) {
                // Calm down back to ambient wander
                self.reaction = BystanderReaction::Wander;
            }
        }

        // Find primary relevant threat
        let perceived_threat = ctx.threats.iter().find(|t| self.can_perceive_threat(t));

        if let Some(threat) = perceived_threat {
            self.panic_timer = 12.0; // Stay agitated for 12 seconds
            self.reaction = self.determine_reaction(threat);
        }
    }

    /// Checks if threat is within vision cone or audible hearing distance.
    pub fn can_perceive_threat(&self, threat: &ThreatStimulus) -> bool {
        let diff = threat.position - self.position;
        let dist = diff.length();

        // Gunfire is always audible within hearing range
        if threat.is_gunfire && dist <= self.senses.hearing_range {
            return true;
        }

        // Being directly aimed at or attacked in close proximity
        if threat.is_aimed_at_me || threat.is_melee_attack {
            if dist <= self.senses.sight_range * 1.2 {
                return true;
            }
        }

        // Standard vision cone
        if dist <= self.senses.sight_range && dist > 1e-4 {
            let dir = diff / dist;
            let cos_angle = self.forward.normalize_or_zero().dot(dir).clamp(-1.0, 1.0);
            let angle = cos_angle.acos();
            if angle <= (self.senses.fov_angle_rad * 0.5) {
                return true;
            }
        }

        false
    }

    /// Decides reaction according to personality archetype.
    fn determine_reaction(&self, threat: &ThreatStimulus) -> BystanderReaction {
        let diff = threat.position - self.position;
        let dist = diff.length();

        match self.personality {
            NpcPersonality::Coward => {
                if threat.is_aimed_at_me && dist < 6.0 {
                    BystanderReaction::HandsUp
                } else if (threat.is_gunfire && dist < 12.0) || dist < 6.0 {
                    BystanderReaction::Cower
                } else {
                    BystanderReaction::Flee {
                        threat_source: threat.position,
                    }
                }
            }
            NpcPersonality::AverageCitizen => {
                if threat.is_aimed_at_me && dist < 5.0 {
                    BystanderReaction::HandsUp
                } else if threat.is_gunfire && dist < 10.0 {
                    BystanderReaction::Cower
                } else {
                    BystanderReaction::Flee {
                        threat_source: threat.position,
                    }
                }
            }
            NpcPersonality::Aggressive => {
                if threat.is_melee_attack || (threat.is_aimed_at_me && dist < 2.5) {
                    BystanderReaction::FightBack {
                        target_id: threat.source_id,
                        is_armed: false,
                    }
                } else if threat.is_gunfire {
                    BystanderReaction::Flee {
                        threat_source: threat.position,
                    }
                } else {
                    BystanderReaction::FightBack {
                        target_id: threat.source_id,
                        is_armed: false,
                    }
                }
            }
            NpcPersonality::ArmedVigilante => {
                if threat.is_gunfire || threat.is_aimed_at_me || threat.is_melee_attack {
                    BystanderReaction::FightBack {
                        target_id: threat.source_id,
                        is_armed: true,
                    }
                } else {
                    BystanderReaction::Flee {
                        threat_source: threat.position,
                    }
                }
            }
        }
    }
}
