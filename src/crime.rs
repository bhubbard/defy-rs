use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Type of criminal infraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrimeType {
    TrafficViolation,
    VehicleTheft,
    Assault,
    ArmedRobbery,
    Murder,
    PoliceAssault,
    OfficerKilled,
    ExplosionTerrorism,
}

impl CrimeType {
    /// Score points added to police wanted level.
    pub fn score_points(&self) -> u32 {
        match self {
            CrimeType::TrafficViolation => 35,
            CrimeType::VehicleTheft => 100,
            CrimeType::Assault => 150,
            CrimeType::ArmedRobbery => 320,
            CrimeType::Murder => 500,
            CrimeType::PoliceAssault => 650,
            CrimeType::OfficerKilled => 1200,
            CrimeType::ExplosionTerrorism => 1800,
        }
    }

    /// Sound loudness in meters (audible hearing radius).
    pub fn auditory_radius(&self) -> f32 {
        match self {
            CrimeType::TrafficViolation => 10.0,
            CrimeType::VehicleTheft => 30.0,      // Alarm blaring
            CrimeType::Assault => 25.0,           // Shouting / blunt hits
            CrimeType::ArmedRobbery => 40.0,      // Screaming
            CrimeType::Murder => 60.0,            // Gunshot or scream
            CrimeType::PoliceAssault => 80.0,     // Gunfire & siren
            CrimeType::OfficerKilled => 90.0,     // High calibre gunfire
            CrimeType::ExplosionTerrorism => 200.0, // Massive blast
        }
    }

    /// Minimum star level immediately assigned upon reporting.
    pub fn minimum_star_level(&self) -> Option<u8> {
        match self {
            CrimeType::PoliceAssault => Some(3),
            CrimeType::OfficerKilled => Some(4),
            CrimeType::ExplosionTerrorism => Some(5),
            _ => None,
        }
    }
}

/// Severity classification of the crime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CrimeSeverity {
    Minor,
    Moderate,
    Felony,
    Capital,
}

impl CrimeType {
    pub fn severity(&self) -> CrimeSeverity {
        match self {
            CrimeType::TrafficViolation => CrimeSeverity::Minor,
            CrimeType::VehicleTheft | CrimeType::Assault => CrimeSeverity::Moderate,
            CrimeType::ArmedRobbery | CrimeType::Murder => CrimeSeverity::Felony,
            CrimeType::PoliceAssault | CrimeType::OfficerKilled | CrimeType::ExplosionTerrorism => {
                CrimeSeverity::Capital
            }
        }
    }
}

/// Record of an observed criminal act.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrimeEvent {
    pub crime_type: CrimeType,
    pub perpetrator_id: u64,
    pub location: Vec3,
    pub timestamp_sec: f64,
    pub is_direct_police_view: bool,
}

/// Sensory stimulus produced in the environment (gunfire, explosion, yelling).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensoryStimulus {
    pub origin: Vec3,
    pub kind: StimulusKind,
    pub intensity_radius: f32,
    pub perpetrator_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StimulusKind {
    VisualCrime(CrimeType),
    GunfireSound,
    ExplosionSound,
    CarAlarm,
    Scream,
}

/// Call status of a civilian witness.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WitnessCallState {
    /// In initial shock / disbelief before reacting.
    Shock { timer: f32 },
    /// Dialing and speaking to 911 dispatch.
    Calling911 { timer: f32, duration: f32 },
    /// Call finished; 911 operator dispatched police.
    Reported,
    /// Silenced / neutralized by player (knocked out, killed, intimidated, or disconnected).
    Silenced,
}

/// Active witness tracking a witnessed crime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveWitness {
    pub witness_id: u64,
    pub crime: CrimeEvent,
    pub state: WitnessCallState,
    pub witness_pos: Vec3,
    pub call_duration: f32,
    pub is_alive: bool,
    pub is_intimidated: bool,
}

/// System managing crime events, witness sensing, and 911 reporting pipelines.
#[derive(Debug, Clone, Default)]
pub struct WitnessSystem {
    pub active_witnesses: Vec<ActiveWitness>,
}

impl WitnessSystem {
    pub fn new() -> Self {
        Self {
            active_witnesses: Vec::new(),
        }
    }

    /// Evaluates if a bystander at `observer_pos` looking in `observer_forward` detects the crime.
    pub fn can_detect(
        observer_pos: Vec3,
        observer_forward: Vec3,
        sight_range: f32,
        fov_angle_rad: f32,
        crime_pos: Vec3,
        crime_type: CrimeType,
        has_line_of_sight: bool,
    ) -> bool {
        let diff = crime_pos - observer_pos;
        let dist = diff.length();

        // Check auditory detection first (sound travels 360 degrees)
        if dist <= crime_type.auditory_radius() {
            return true;
        }

        // Visual detection requires line of sight and within field-of-view cone
        if dist <= sight_range && has_line_of_sight {
            if dist < 1e-4 {
                return true;
            }
            let dir_to_crime = diff / dist;
            let cos_angle = observer_forward.normalize_or_zero().dot(dir_to_crime).clamp(-1.0, 1.0);
            let angle = cos_angle.acos();
            if angle <= (fov_angle_rad * 0.5) {
                return true;
            }
        }

        false
    }

    /// Register a newly detected crime by an NPC witness.
    pub fn register_witness(
        &mut self,
        witness_id: u64,
        witness_pos: Vec3,
        crime: CrimeEvent,
        call_duration: f32,
    ) {
        // Prevent duplicate call from same witness for same crime
        if self.active_witnesses.iter().any(|w| w.witness_id == witness_id) {
            return;
        }

        self.active_witnesses.push(ActiveWitness {
            witness_id,
            crime,
            state: WitnessCallState::Shock { timer: 0.8 },
            witness_pos,
            call_duration,
            is_alive: true,
            is_intimidated: false,
        });
    }

    /// Silence an active witness (e.g. player aimed gun, punched, knocked out, or killed them).
    pub fn silence_witness(&mut self, witness_id: u64) -> bool {
        if let Some(w) = self.active_witnesses.iter_mut().find(|w| w.witness_id == witness_id) {
            if matches!(w.state, WitnessCallState::Shock { .. } | WitnessCallState::Calling911 { .. }) {
                w.state = WitnessCallState::Silenced;
                return true;
            }
        }
        false
    }

    /// Updates active 911 phone calls. Returns a list of completed crime reports dispatched to police.
    pub fn update(&mut self, dt: f32) -> Vec<CrimeEvent> {
        let mut dispatched_crimes = Vec::new();

        for w in self.active_witnesses.iter_mut() {
            if !w.is_alive || w.is_intimidated {
                w.state = WitnessCallState::Silenced;
                continue;
            }

            match &mut w.state {
                WitnessCallState::Shock { timer } => {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        w.state = WitnessCallState::Calling911 {
                            timer: 0.0,
                            duration: w.call_duration,
                        };
                    }
                }
                WitnessCallState::Calling911 { timer, duration } => {
                    *timer += dt;
                    if *timer >= *duration {
                        w.state = WitnessCallState::Reported;
                        dispatched_crimes.push(w.crime.clone());
                    }
                }
                WitnessCallState::Reported | WitnessCallState::Silenced => {}
            }
        }

        // Clean up finalized witnesses
        self.active_witnesses
            .retain(|w| !matches!(w.state, WitnessCallState::Reported | WitnessCallState::Silenced));

        dispatched_crimes
    }
}
