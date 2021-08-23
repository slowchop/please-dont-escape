mod game;
mod input;
mod menus;
mod splash;

use crate::game::Game;
use crate::menus::MainMenu;
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
    let initial_app_state = match env::args().skip(1).next().as_deref() {
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
        .add_state(initial_app_state)
        .add_plugin(SplashScreen)
        .add_plugin(MainMenu)
        .add_plugin(Game)
        .run();
}

