use glam::Vec3;
use defy_rs::{
    DispatchComposition, DispatchUnitType, PoliceUnit, PursuitAiState, PursuitConfig,
    WantedLevelSystem, WantedStatus,
};

#[test]
fn test_star_progression_by_score() {
    let mut system = WantedLevelSystem::default();
    assert_eq!(system.stars, 0);

    // 100 pts -> 1 star
    system.add_crime_score(120, Vec3::new(10.0, 0.0, 10.0));
    assert_eq!(system.stars, 1);
    assert_eq!(system.status, WantedStatus::ActivePursuit);

    // Additional 400 pts (total 520) -> 3 stars
    system.add_crime_score(400, Vec3::new(20.0, 0.0, 20.0));
    assert_eq!(system.stars, 3);

    // Additional 2000 pts (total 2520) -> 6 stars
    system.add_crime_score(2000, Vec3::new(30.0, 0.0, 30.0));
    assert_eq!(system.stars, 6);
}

#[test]
fn test_dispatch_composition_per_star() {
    let comp1 = DispatchComposition::for_stars(1);
    assert_eq!(comp1.max_active_units, 2);
    assert_eq!(comp1.roadblock_probability, 0.0);
    assert!(comp1.units.contains(&DispatchUnitType::StandardCruiser));

    let comp3 = DispatchComposition::for_stars(3);
    assert_eq!(comp3.max_active_units, 6);
    assert!(comp3.units.contains(&DispatchUnitType::PoliceHelicopter));

    let comp6 = DispatchComposition::for_stars(6);
    assert_eq!(comp6.max_active_units, 14);
    assert!(comp6.units.contains(&DispatchUnitType::MilitaryRhinoTank));
    assert_eq!(comp6.roadblock_probability, 1.0);
}

#[test]
fn test_line_of_sight_and_decay_cycle() {
    let mut system = WantedLevelSystem::default();
    system.set_minimum_stars(1, Vec3::ZERO);
    assert_eq!(system.stars, 1);

    let suspect_pos = Vec3::new(50.0, 0.0, 50.0);
    let mut unit = PoliceUnit::new(1, DispatchUnitType::StandardCruiser, Vec3::ZERO);
    unit.has_line_of_sight = true;

    // Tick with active visual contact
    system.update(1.0, suspect_pos, 5.0, &mut [unit.clone()]);
    assert_eq!(system.status, WantedStatus::ActivePursuit);
    assert_eq!(system.decay_timer, 20.0);

    // Unit loses visual contact
    unit.has_line_of_sight = false;
    system.update(1.0, suspect_pos, 5.0, &mut [unit.clone()]);
    assert_eq!(system.status, WantedStatus::SearchingFlashing);
    assert_eq!(system.decay_timer, 19.0);

    // Advance 20 seconds out of sight: wanted stars should decay to 0
    for _ in 0..20 {
        system.update(1.0, suspect_pos, 5.0, &mut [unit.clone()]);
    }
    assert_eq!(system.stars, 0);
    assert_eq!(system.status, WantedStatus::Clean);
}

#[test]
fn test_reacquisition_resets_decay() {
    let mut system = WantedLevelSystem::default();
    system.set_minimum_stars(2, Vec3::ZERO);

    let suspect_pos = Vec3::new(10.0, 0.0, 10.0);
    let mut unit = PoliceUnit::new(1, DispatchUnitType::StandardCruiser, Vec3::ZERO);
    unit.has_line_of_sight = false;

    // Flash for 10 seconds
    for _ in 0..10 {
        system.update(1.0, suspect_pos, 5.0, &mut [unit.clone()]);
    }
    assert_eq!(system.status, WantedStatus::SearchingFlashing);
    assert!(system.decay_timer < 35.0);

    // Regain visual contact
    unit.has_line_of_sight = true;
    system.update(0.1, suspect_pos, 5.0, &mut [unit.clone()]);
    assert_eq!(system.status, WantedStatus::ActivePursuit);
    assert_eq!(system.decay_timer, 35.0); // Reset to full duration
}

#[test]
fn test_busted_arrest_mechanic() {
    let mut system = WantedLevelSystem::new(PursuitConfig {
        arrest_distance_threshold: 4.0,
        arrest_timer_required: 2.0,
        ..Default::default()
    });
    system.set_minimum_stars(1, Vec3::ZERO);

    let suspect_pos = Vec3::ZERO;
    let mut units = [PoliceUnit::new(1, DispatchUnitType::StandardCruiser, Vec3::new(2.0, 0.0, 0.0))];
    units[0].has_line_of_sight = true;

    // Suspect is stopped (speed 0.0) next to cop
    system.update(1.0, suspect_pos, 0.0, &mut units);
    assert_eq!(units[0].state, PursuitAiState::Arrest);
    assert_eq!(system.arrest_progress, 1.0);
    assert!(!system.is_busted);

    // Second tick finishes arrest
    system.update(1.1, suspect_pos, 0.0, &mut units);
    assert!(system.is_busted);
}

#[test]
fn test_deadly_force_at_high_wanted_level() {
    let mut system = WantedLevelSystem::default();
    system.set_minimum_stars(4, Vec3::ZERO);

    let suspect_pos = Vec3::ZERO;
    let mut units = [PoliceUnit::new(1, DispatchUnitType::SwatEnforcerVan, Vec3::new(15.0, 0.0, 0.0))];
    units[0].has_line_of_sight = true;

    system.update(0.1, suspect_pos, 0.0, &mut units);
    // At 4 stars, state must be DeadlyForce regardless of distance
    assert_eq!(units[0].state, PursuitAiState::DeadlyForce);
}
