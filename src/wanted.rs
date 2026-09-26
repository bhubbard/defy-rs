use glam::Vec3;
use serde::{Deserialize, Serialize};

/// 1 to 6 stars wanted rating.
pub type StarLevel = u8;

/// Police vehicle and reinforcement unit types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DispatchUnitType {
    StandardCruiser,
    UnmarkedCruiser,
    SwatEnforcerVan,
    PoliceHelicopter,
    FibTacticalSuv,
    RoadblockWithSpikes,
    ArmyBarracksTruck,
    MilitaryRhinoTank,
}

/// Unit composition and reinforcement frequency for a given star tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchComposition {
    pub units: Vec<DispatchUnitType>,
    pub spawn_interval_sec: f32,
    pub max_active_units: usize,
    pub roadblock_probability: f32,
    pub search_radius: f32,
    pub decay_duration_sec: f32,
}

impl DispatchComposition {
    pub fn for_stars(stars: StarLevel) -> Self {
        match stars {
            0 => Self {
                units: vec![],
                spawn_interval_sec: 999.0,
                max_active_units: 0,
                roadblock_probability: 0.0,
                search_radius: 0.0,
                decay_duration_sec: 0.0,
            },
            1 => Self {
                units: vec![DispatchUnitType::StandardCruiser],
                spawn_interval_sec: 25.0,
                max_active_units: 2,
                roadblock_probability: 0.0,
                search_radius: 70.0,
                decay_duration_sec: 20.0,
            },
            2 => Self {
                units: vec![
                    DispatchUnitType::StandardCruiser,
                    DispatchUnitType::StandardCruiser,
                    DispatchUnitType::UnmarkedCruiser,
                ],
                spawn_interval_sec: 18.0,
                max_active_units: 4,
                roadblock_probability: 0.15,
                search_radius: 120.0,
                decay_duration_sec: 35.0,
            },
            3 => Self {
                units: vec![
                    DispatchUnitType::StandardCruiser,
                    DispatchUnitType::SwatEnforcerVan,
                    DispatchUnitType::PoliceHelicopter,
                ],
                spawn_interval_sec: 12.0,
                max_active_units: 6,
                roadblock_probability: 0.40,
                search_radius: 180.0,
                decay_duration_sec: 50.0,
            },
            4 => Self {
                units: vec![
                    DispatchUnitType::SwatEnforcerVan,
                    DispatchUnitType::PoliceHelicopter,
                    DispatchUnitType::RoadblockWithSpikes,
                ],
                spawn_interval_sec: 9.0,
                max_active_units: 8,
                roadblock_probability: 0.70,
                search_radius: 250.0,
                decay_duration_sec: 70.0,
            },
            5 => Self {
                units: vec![
                    DispatchUnitType::FibTacticalSuv,
                    DispatchUnitType::PoliceHelicopter,
                    DispatchUnitType::SwatEnforcerVan,
                    DispatchUnitType::RoadblockWithSpikes,
                ],
                spawn_interval_sec: 6.0,
                max_active_units: 10,
                roadblock_probability: 0.85,
                search_radius: 350.0,
                decay_duration_sec: 90.0,
            },
            _ => Self {
                // 6 Stars: Military Martial Law
                units: vec![
                    DispatchUnitType::MilitaryRhinoTank,
                    DispatchUnitType::ArmyBarracksTruck,
                    DispatchUnitType::FibTacticalSuv,
                    DispatchUnitType::PoliceHelicopter,
                    DispatchUnitType::RoadblockWithSpikes,
                ],
                spawn_interval_sec: 4.5,
                max_active_units: 14,
                roadblock_probability: 1.0,
                search_radius: 500.0,
                decay_duration_sec: 120.0,
            },
        }
    }
}

/// Pursuit AI state of an individual law enforcement officer/unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PursuitAiState {
    #[default]
    Patrol,
    Investigate,
    Pursue,
    Arrest,
    DeadlyForce,
}

/// Individual responding law enforcement unit in the simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoliceUnit {
    pub id: u64,
    pub unit_type: DispatchUnitType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub state: PursuitAiState,
    pub has_line_of_sight: bool,
    pub distance_to_suspect: f32,
    pub is_alive: bool,
}

impl PoliceUnit {
    pub fn new(id: u64, unit_type: DispatchUnitType, position: Vec3) -> Self {
        Self {
            id,
            unit_type,
            position,
            velocity: Vec3::ZERO,
            state: PursuitAiState::Patrol,
            has_line_of_sight: false,
            distance_to_suspect: f32::MAX,
            is_alive: true,
        }
    }
}

/// Status of the wanted level system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WantedStatus {
    #[default]
    Clean,
    /// Police have active visual contact and are actively chasing.
    ActivePursuit,
    /// Police lost visual contact; search radius is active and stars are flashing as decay ticks down.
    SearchingFlashing,
}

/// Configuration constants for police pursuit tuning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PursuitConfig {
    pub score_1_star: u32,
    pub score_2_stars: u32,
    pub score_3_stars: u32,
    pub score_4_stars: u32,
    pub score_5_stars: u32,
    pub score_6_stars: u32,
    pub arrest_distance_threshold: f32,
    pub arrest_timer_required: f32,
}

impl Default for PursuitConfig {
    fn default() -> Self {
        Self {
            score_1_star: 100,
            score_2_stars: 250,
            score_3_stars: 500,
            score_4_stars: 900,
            score_5_stars: 1500,
            score_6_stars: 2500,
            arrest_distance_threshold: 4.0,
            arrest_timer_required: 3.0,
        }
    }
}

/// Core police wanted-level manager.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WantedLevelSystem {
    pub stars: StarLevel,
    pub crime_score: u32,
    pub status: WantedStatus,
    pub last_known_position: Vec3,
    pub decay_timer: f32,
    pub spawn_cooldown: f32,
    pub config: PursuitConfig,
    pub arrest_progress: f32,
    pub is_busted: bool,
}

impl Default for WantedLevelSystem {
    fn default() -> Self {
        Self {
            stars: 0,
            crime_score: 0,
            status: WantedStatus::Clean,
            last_known_position: Vec3::ZERO,
            decay_timer: 0.0,
            spawn_cooldown: 0.0,
            config: PursuitConfig::default(),
            arrest_progress: 0.0,
            is_busted: false,
        }
    }
}

impl WantedLevelSystem {
    pub fn new(config: PursuitConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Add crime score points and evaluate star progression.
    pub fn add_crime_score(&mut self, points: u32, suspect_pos: Vec3) {
        self.crime_score = self.crime_score.saturating_add(points);
        let new_stars = self.calculate_stars_from_score(self.crime_score);
        if new_stars > self.stars {
            self.stars = new_stars;
        }

        if self.stars > 0 {
            self.status = WantedStatus::ActivePursuit;
            self.last_known_position = suspect_pos;
            self.decay_timer = DispatchComposition::for_stars(self.stars).decay_duration_sec;
        }
    }

    /// Set minimum star level directly (e.g. killing cop gives automatic 3 stars).
    pub fn set_minimum_stars(&mut self, min_stars: StarLevel, suspect_pos: Vec3) {
        let min_stars = min_stars.clamp(1, 6);
        if min_stars > self.stars {
            self.stars = min_stars;
            let req_score = self.score_for_star_level(min_stars);
            if self.crime_score < req_score {
                self.crime_score = req_score;
            }
        }
        self.status = WantedStatus::ActivePursuit;
        self.last_known_position = suspect_pos;
        self.decay_timer = DispatchComposition::for_stars(self.stars).decay_duration_sec;
    }

    /// Clears wanted level completely (Pay 'n' Spray or successful escape).
    pub fn clear(&mut self) {
        self.stars = 0;
        self.crime_score = 0;
        self.status = WantedStatus::Clean;
        self.decay_timer = 0.0;
        self.arrest_progress = 0.0;
        self.is_busted = false;
    }

    pub fn current_composition(&self) -> DispatchComposition {
        DispatchComposition::for_stars(self.stars)
    }

    fn calculate_stars_from_score(&self, score: u32) -> StarLevel {
        if score >= self.config.score_6_stars {
            6
        } else if score >= self.config.score_5_stars {
            5
        } else if score >= self.config.score_4_stars {
            4
        } else if score >= self.config.score_3_stars {
            3
        } else if score >= self.config.score_2_stars {
            2
        } else if score >= self.config.score_1_star {
            1
        } else {
            0
        }
    }

    fn score_for_star_level(&self, stars: StarLevel) -> u32 {
        match stars {
            1 => self.config.score_1_star,
            2 => self.config.score_2_stars,
            3 => self.config.score_3_stars,
            4 => self.config.score_4_stars,
            5 => self.config.score_5_stars,
            _ => self.config.score_6_stars,
        }
    }

    /// Primary system update. Evaluates visual contact across police units,
    /// decrements flashing decay timer if out of sight, and checks arrest condition.
    pub fn update(
        &mut self,
        dt: f32,
        suspect_pos: Vec3,
        suspect_speed: f32,
        units: &mut [PoliceUnit],
    ) {
        if self.stars == 0 {
            self.status = WantedStatus::Clean;
            return;
        }

        let composition = self.current_composition();

        // Check if ANY active police unit has line of sight to suspect
        let mut visual_contact = false;
        let mut closest_unit_dist = f32::MAX;
        let mut arresting_unit_present = false;

        for unit in units.iter_mut() {
            if !unit.is_alive {
                continue;
            }

            let dist = (unit.position - suspect_pos).length();
            unit.distance_to_suspect = dist;

            if unit.has_line_of_sight {
                visual_contact = true;
                if dist < closest_unit_dist {
                    closest_unit_dist = dist;
                }
            }

            // Update individual AI pursuit behavior
            unit.state = self.evaluate_unit_ai(
                unit,
                dist,
                suspect_pos,
                suspect_speed,
                composition.search_radius,
            );

            if unit.state == PursuitAiState::Arrest && dist <= self.config.arrest_distance_threshold {
                arresting_unit_present = true;
            }
        }

        if visual_contact {
            // Visual contact maintained: update last known position and reset decay timer
            self.status = WantedStatus::ActivePursuit;
            self.last_known_position = suspect_pos;
            self.decay_timer = composition.decay_duration_sec;
        } else {
            // No unit has line of sight: stars start flashing and decay timer ticks
            self.status = WantedStatus::SearchingFlashing;
            self.decay_timer -= dt;

            if self.decay_timer <= 0.0 {
                // Decay by one star level
                self.stars = self.stars.saturating_sub(1);
                if self.stars > 0 {
                    self.decay_timer = DispatchComposition::for_stars(self.stars).decay_duration_sec;
                } else {
                    self.clear();
                    return;
                }
            }
        }

        // Handle Busted / Arrest logic at 1-2 stars when stationary
        if arresting_unit_present && suspect_speed < 1.0 && self.stars <= 2 {
            self.arrest_progress += dt;
            if self.arrest_progress >= self.config.arrest_timer_required {
                self.is_busted = true;
            }
        } else {
            self.arrest_progress = (self.arrest_progress - dt * 2.0).max(0.0);
        }

        // Spawn timer cooldown
        self.spawn_cooldown = (self.spawn_cooldown - dt).max(0.0);
    }

    /// Evaluates pursuit AI state for an individual unit.
    fn evaluate_unit_ai(
        &self,
        unit: &PoliceUnit,
        dist: f32,
        _suspect_pos: Vec3,
        suspect_speed: f32,
        search_radius: f32,
    ) -> PursuitAiState {
        if self.stars == 0 {
            return PursuitAiState::Patrol;
        }

        if unit.has_line_of_sight {
            // At 3+ stars, police always use deadly force
            if self.stars >= 3 {
                return PursuitAiState::DeadlyForce;
            }

            // At 1-2 stars, attempt arrest if suspect is cornered and stopped
            if dist <= self.config.arrest_distance_threshold && suspect_speed < 1.5 {
                PursuitAiState::Arrest
            } else {
                PursuitAiState::Pursue
            }
        } else {
            // No direct sight: if within search radius of last known position, investigate
            let dist_to_last_known = (unit.position - self.last_known_position).length();
            if dist_to_last_known <= search_radius {
                PursuitAiState::Investigate
            } else {
                PursuitAiState::Patrol
            }
        }
    }
}
