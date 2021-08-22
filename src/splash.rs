use bevy::prelude::*;
use crate::game::AppState;

pub struct SplashTimer(Timer);

pub fn splash_setup_system(mut commands: Commands) {
    commands.insert_resource(SplashTimer(Timer::from_seconds(2.0, false)))
}

pub fn splash_timer_system(time: Res<Time>, mut timer: ResMut<SplashTimer>, mut state: ResMut<State<AppState>>) {
    if timer.0.tick(time.delta()).just_finished() {
        debug!("Splash timer done.");
        state.set(AppState::MainMenu);
    }
}

pub fn splash_exit_system(keys: Res<Input<KeyCode>>, mut state: ResMut<State<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        debug!("Splash escaped.");
        state.set(AppState::MainMenu);
    }
}