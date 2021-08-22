use crate::input::input_exit_system;
use crate::setup::setup_system;
use crate::splash::{splash_setup_system, splash_timer_system, splash_exit_system};
use bevy::app::{AppExit, Events};
use bevy::core::{FixedTimestep, FixedTimesteps};
use bevy::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    AskPlayerName,
    MainMenu,
    InGame,
}

pub struct PleaseDontEscape;

impl Plugin for PleaseDontEscape {
    fn build(&self, app: &mut AppBuilder) {
        app
            // State enum
            .add_state(AppState::SplashScreen)
            // Load assets

            .add_startup_system(setup_system.system())
            //
            .add_system_set(
                SystemSet::on_enter(AppState::SplashScreen).with_system(splash_setup_system.system()),
            )
            .add_system_set(SystemSet::on_update(AppState::SplashScreen).with_system(splash_timer_system.system()))
            .add_system_set(SystemSet::on_update(AppState::SplashScreen).with_system(splash_exit_system.system()))
            // .add_startup_system(spawn_entities_system.system())
            //
            // .add_system(hello_world_system.system())
            // .add_system(input_exit_system.system())
            // Fixed timestamp systems!
            // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
                    .with_system(fixed_system.system()),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

fn fixed_system() {
    // println!("fixed");
}
