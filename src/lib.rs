mod editor;
mod game;
mod input;
mod map;
mod menus;
mod path;
mod player;
mod position;
mod wires;

use crate::editor::Editor;
use crate::game::Game;
use crate::menus::MainMenu;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use slowchop::{SplashScreen, SplashScreenConfig};
use std::env;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    AskPlayerName,
    MainMenu,
    InGame,
    Editor,
}

#[wasm_bindgen]
pub fn run() {
    let initial_app_state = match env::args().skip(1).next().as_deref() {
        Some("solo") => AppState::InGame,
        Some("editor") => AppState::Editor,
        Some(x) => panic!("Unknown argument: {}", x),
        None => AppState::MainMenu,
    };

    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        title: "Please Do Not Escape".to_string(),
        width: 1920.0,
        height: 1080.0,
        resizable: false,
        ..Default::default()
    })
    // .add_plugin(LogDiagnosticsPlugin::default())
    // .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .insert_resource(bevy::log::LogSettings {
        level: bevy::log::Level::DEBUG,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app //
        .add_plugin(EguiPlugin)
        .add_state(initial_app_state)
        .insert_resource(SplashScreenConfig {
            timer: Timer::from_seconds(2.0, false),
            image: "menus/logo.png".to_string(),
        })
        .add_plugin(SplashScreen)
        // .add_plugin(MainMenu)
        // .add_plugin(Game)
        // .add_plugin(Editor)
        .run();
}
