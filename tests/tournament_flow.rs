use shiphappens::scoring::ci::{composite_score, CompositeInput};
use shiphappens::tournament::{TournamentConfig, TournamentDirector, TournamentPhase};

#[test]
fn tournament_bootstraps_four_solo_slots() {
    let config = TournamentConfig::default();
    let director = TournamentDirector::bootstrap(&config);
    assert_eq!(director.slots.len(), 4);
    assert_eq!(director.phase, TournamentPhase::Lobby);
}

#[test]
fn composite_score_weights_cleared_room() {
    let score = composite_score(CompositeInput {
        cleared: true,
        clear_time_secs: 30.0,
        fastest_clear_secs: 30.0,
        efficiency: 90.0,
        cooperation: 50.0,
        partial_progress: 0.0,
    });
    assert!(score > 80.0);
}
