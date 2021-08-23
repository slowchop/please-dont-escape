mod game;
mod input;
mod menus;
mod setup;
mod splash;

use crate::game::Game;
use crate::menus::MainMenu;
use crate::setup::setup_system;
use crate::splash::SplashScreen;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::env;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    AskPlayerName,
    MainMenu,
    InGame,
}

fn main() {
    let app_state = match env::args().skip(1).next().as_deref() {
        Some("solo") => AppState::InGame,
        Some(x) => panic!("Unknown argument: {}", x),
        None => AppState::SplashScreen,
    };

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
        .add_plugin(EguiPlugin)
        //
        .add_state(app_state)
        //
        // Splash
        .add_plugin(SplashScreen)
        // Load assets (after splash screen assets are loaded hopefully!)
        .add_startup_system(setup_system.system())
        .add_plugin(MainMenu)
        .add_plugin(Game)
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

