mod background;
pub mod crater;

// app.add_systems(
//     OnEnter(AppState::CreateNewStory),
//     ((
//         (setup_crater, apply_deferred).chain(),
//         (setup_crater_elements, apply_deferred).chain(),
//         (setup_crater_ants, apply_deferred).chain(),
//     )
//         .chain()
//         .before(finalize_startup)
//         .after(setup_settings))
//     .chain(),
// );

// app.add_systems(
//     OnEnter(AppState::FinishSetup),
//     (
//         (setup_crater_grid, apply_deferred).chain(),
//         (setup_background, apply_deferred).chain(),
//     )
//         .chain(),
// );

// app.add_systems(
//     OnEnter(AppState::Cleanup),
//     (teardown_background, teardown_crater),
// );
