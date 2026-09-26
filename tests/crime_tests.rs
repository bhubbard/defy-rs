use glam::Vec3;
use defy_rs::{
    CrimeEvent, CrimeSeverity, CrimeType, WitnessCallState, WitnessSystem,
};

#[test]
fn test_crime_severities_and_scores() {
    assert_eq!(CrimeType::TrafficViolation.severity(), CrimeSeverity::Minor);
    assert_eq!(CrimeType::VehicleTheft.severity(), CrimeSeverity::Moderate);
    assert_eq!(CrimeType::Murder.severity(), CrimeSeverity::Felony);
    assert_eq!(CrimeType::OfficerKilled.severity(), CrimeSeverity::Capital);

    assert_eq!(CrimeType::PoliceAssault.minimum_star_level(), Some(3));
    assert_eq!(CrimeType::OfficerKilled.minimum_star_level(), Some(4));
    assert_eq!(CrimeType::ExplosionTerrorism.minimum_star_level(), Some(5));
}

#[test]
fn test_sensory_detection_vision_cone() {
    let observer_pos = Vec3::ZERO;
    let observer_forward = Vec3::Z; // Facing +Z
    let sight_range = 30.0;
    let fov = 90.0_f32.to_radians();

    // Target directly ahead at 15m
    assert!(WitnessSystem::can_detect(
        observer_pos,
        observer_forward,
        sight_range,
        fov,
        Vec3::new(0.0, 0.0, 15.0),
        CrimeType::TrafficViolation,
        true
    ));

    // Target behind observer (180 degrees) for quiet crime (traffic violation has 10m hearing radius, target is at 20m)
    assert!(!WitnessSystem::can_detect(
        observer_pos,
        observer_forward,
        sight_range,
        fov,
        Vec3::new(0.0, 0.0, -20.0),
        CrimeType::TrafficViolation,
        true
    ));
}

#[test]
fn test_sensory_detection_audio_omnidirectional() {
    let observer_pos = Vec3::ZERO;
    let observer_forward = Vec3::Z;
    let sight_range = 30.0;
    let fov = 90.0_f32.to_radians();

    // Massive explosion behind the witness at 120m away (auditory radius is 200m)
    assert!(WitnessSystem::can_detect(
        observer_pos,
        observer_forward,
        sight_range,
        fov,
        Vec3::new(0.0, 0.0, -120.0),
        CrimeType::ExplosionTerrorism,
        false // No visual line of sight
    ));
}

#[test]
fn test_911_call_pipeline_and_dispatch() {
    let mut system = WitnessSystem::new();
    let crime = CrimeEvent {
        crime_type: CrimeType::VehicleTheft,
        perpetrator_id: 42,
        location: Vec3::new(5.0, 0.0, 5.0),
        timestamp_sec: 100.0,
        is_direct_police_view: false,
    };

    system.register_witness(101, Vec3::new(10.0, 0.0, 10.0), crime.clone(), 3.0);
    assert_eq!(system.active_witnesses.len(), 1);

    // Initial shock phase (0.8s)
    let dispatched = system.update(0.5);
    assert!(dispatched.is_empty());
    assert!(matches!(
        system.active_witnesses[0].state,
        WitnessCallState::Shock { .. }
    ));

    // Transition into Calling 911
    let dispatched = system.update(0.5);
    assert!(dispatched.is_empty());
    assert!(matches!(
        system.active_witnesses[0].state,
        WitnessCallState::Calling911 { .. }
    ));

    // Complete the 3.0s call
    let dispatched = system.update(3.2);
    assert_eq!(dispatched.len(), 1);
    assert_eq!(dispatched[0].crime_type, CrimeType::VehicleTheft);
    // Finished witness is pruned
    assert_eq!(system.active_witnesses.len(), 0);
}

#[test]
fn test_silence_witness_cancels_call() {
    let mut system = WitnessSystem::new();
    let crime = CrimeEvent {
        crime_type: CrimeType::Murder,
        perpetrator_id: 7,
        location: Vec3::ZERO,
        timestamp_sec: 50.0,
        is_direct_police_view: false,
    };

    system.register_witness(99, Vec3::new(4.0, 0.0, 0.0), crime, 4.0);

    // Advance into calling state
    system.update(1.0);
    assert!(matches!(
        system.active_witnesses[0].state,
        WitnessCallState::Calling911 { .. }
    ));

    // Player intimidates or silences witness before 911 operator picks up
    let silenced = system.silence_witness(99);
    assert!(silenced);

    // Further ticks produce NO dispatched crime events
    let dispatched = system.update(5.0);
    assert!(dispatched.is_empty());
    assert_eq!(system.active_witnesses.len(), 0);
}
