mod input;
mod menus;
mod setup;
mod splash;

use crate::input::exit_on_escape_key;
use crate::menus::MainMenu;
use crate::setup::setup_system;
use crate::splash::SplashScreen;
use bevy::app::{AppExit, Events};
use bevy::core::{FixedTimestep, FixedTimesteps};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    AskPlayerName,
    MainMenu,
    InGame,
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Please Don't Escape".to_string(),
            width: 1920.0,
            height: 1080.0,
            resizable: false,
            ..Default::default()
        })
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        //
        .add_state(AppState::SplashScreen)
        //
        // Splash
        .add_plugin(SplashScreen)
        // Load assets (after splash screen assets are loaded hopefully!)
        .add_startup_system(setup_system.system())
        // Main Menu
        .add_plugin(MainMenu)
        // .add_system_set(
        //     SystemSet::on_enter(AppState::MainMenu).with_system(main_menu_setup.system()),
        // )
        // // .add_startup_system(spawn_entities_system.system())
        // //
        // // .add_system(hello_world_system.system())
        // // .add_system(input_exit_system.system())
        // // Fixed timestamp systems!
        // // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
        // .add_stage_after(
        //     CoreStage::Update,
        //     FixedUpdateStage,
        //     SystemStage::parallel()
        //         .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
        //         .with_system(fixed_system.system()),
        // )
        .run();
}

fn fixed_system() {}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;
