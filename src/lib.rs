mod game;
mod input;
mod map;
mod menus;
mod splash;
mod position;

use crate::game::Game;
use crate::menus::MainMenu;
use crate::splash::SplashScreen;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::env;
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    AskPlayerName,
    MainMenu,
    InGame,
}

#[wasm_bindgen]
pub fn run() {
    let initial_app_state = match env::args().skip(1).next().as_deref() {
        Some("solo") => AppState::InGame,
        Some(x) => panic!("Unknown argument: {}", x),
        None => AppState::SplashScreen,
    };

    let mut app = App::build();
    app
        .insert_resource(WindowDescriptor {
            title: "Please Don't Escape".to_string(),
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

    app
        .add_plugin(EguiPlugin)
        .add_state(initial_app_state)
        .add_plugin(SplashScreen)
        .add_plugin(MainMenu)
        .add_plugin(Game)
        .run();
}
