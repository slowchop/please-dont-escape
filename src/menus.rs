use crate::input::exit_on_escape_key;
use crate::AppState;
use bevy::app::{AppExit, Events};
use bevy::prelude::*;
use bevy_egui::egui::{FontDefinitions, FontFamily, Layout};
use bevy_egui::{egui, EguiContext};

const LOGO_ID: u64 = 0;

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            .insert_resource(ClearColor(Color::hex("BABCAD").unwrap()))
            .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup.system()))
            .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(next.system()))
            .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(cleanup.system()));
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(camera);

    let material = materials.add(asset_server.load("menus/logo.png").into());
    commands.spawn_bundle(SpriteBundle {
        material,
        ..Default::default()
    });
}

fn cleanup(mut commands: Commands, entities: Query<Entity>) {
    debug!("Cleanup.");
    for sprite in entities.iter() {
        commands.entity(sprite).despawn_recursive();
    }
}

pub fn next(
    mut state: ResMut<State<AppState>>,
    keys: Res<Input<KeyCode>>,
    mut mouse_button: ResMut<Input<MouseButton>>,
) {
    for _ in keys.get_just_pressed() {
        state.set(AppState::InGame).unwrap();
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        mouse_button.reset(MouseButton::Left);
        state.set(AppState::InGame).unwrap();
    }
}
