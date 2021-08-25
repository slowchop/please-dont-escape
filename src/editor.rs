use crate::game::CELL_SIZE;
use crate::position::{GridPosition, Position};
use crate::AppState;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::utils::StableHashMap;
use bevy_egui::egui::{FontDefinitions, Ui};
use bevy_egui::{egui, EguiContext};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::fs::File;
use std::io::Write;
use std::ops::{Add, Deref};
use std::path::PathBuf;

pub struct Editor;

impl Plugin for Editor {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            .insert_resource(Drag(Vec2::default()))
            .insert_resource(Map::new())
            .insert_resource(UiFilename("level1".into()))
            .insert_resource(Mode::Add)
            .insert_resource(UiItem::Wall)
            //
            .add_system_set(SystemSet::on_enter(AppState::Editor).with_system(setup.system()))
            .add_system_set(
                SystemSet::on_update(AppState::Editor)
                    .with_system(ui.system())
                    .with_system(camera_to_selection.system())
                    .with_system(click_add.system())
                    .with_system(drag_diff.system())
                    .with_system(drag.system()),
            );
    }
}

struct Selection;

fn setup(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let fonts = FontDefinitions::default();
    egui_context.ctx().set_fonts(fonts);

    let style: egui::Style = egui::Style::default();
    egui_context.ctx().set_style(style);

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());

    let selection = materials.add(asset_server.load("cells/selection.png").into());
    let grid_pos = GridPosition::zero();
    commands
        .spawn_bundle(SpriteBundle {
            material: selection,
            transform: Position::from(grid_pos).to_transform(),
            ..Default::default()
        })
        .insert(Selection);
}

fn ui(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut ui_filename: ResMut<UiFilename>,
    mut mode: ResMut<Mode>,
    mut ui_item: ResMut<UiItem>,
    mut map: ResMut<Map>,
) {
    egui::Window::new("Editor")
        .default_width(200.0)
        .show(egui_context.ctx(), |ui| {
            ui.button("New");

            ui.horizontal(|ui| {
                ui.label("Filename:");
                ui.text_edit_singleline(&mut ui_filename.0);
            });

            ui.horizontal(|ui| {
                let path = PathBuf::from(format!("assets/maps/{}.json", &ui_filename.0));
                if ui.button("Load").clicked() {
                    info!("Loading from {:?}", &path);
                    let mut f = File::open(&path).expect("Could not open file for reading.");
                    *map = serde_json::from_reader(f).expect("Could not read from file.");

                    for item in &map.items {
                        let pos = item.position().into();
                        add_item(&mut commands, &mut materials, &asset_server, &pos, &*ui_item);
                    }
                };
                if ui.button("Save").clicked() {
                    info!("Saving to {:?}", &path);
                    let serialized = serde_json::to_vec_pretty(&*map).unwrap();
                    let mut f = File::create(&path).expect("Could not open file for writing.");
                    f.write_all(&serialized).expect("Could not write to file.");
                }
            });

            ui.separator();

            ui.heading("Mode");
            ui.horizontal_wrapped(|ui| {
                select_mode(ui, "Add", &mut mode, Mode::Add);
                select_mode(ui, "Select", &mut mode, Mode::Select);
            });
            ui.heading("Item");
            ui.horizontal_wrapped(|ui| {
                select_item(ui, "Wall", &mut ui_item, UiItem::Wall);
                select_item(ui, "Warden Spawn", &mut ui_item, UiItem::Warden);
                select_item(ui, "Prisoner Spawn", &mut ui_item, UiItem::Prisoner);
                select_item(ui, "Security Door", &mut ui_item, UiItem::Door);
                select_item(ui, "Exit", &mut ui_item, UiItem::Exit);
                select_item(ui, "Wire", &mut ui_item, UiItem::Wire);
            });
        });
}

// TODO: Work out how to make generic
fn select_item(ui: &mut Ui, title: &str, item: &mut ResMut<UiItem>, new_item: UiItem) {
    if ui.selectable_label(**item == new_item, title).clicked() {
        **item = new_item;
    };
}

fn select_mode(ui: &mut Ui, title: &str, item: &mut ResMut<Mode>, new_item: Mode) {
    if ui.selectable_label(**item == new_item, title).clicked() {
        **item = new_item;
    };
}

fn camera_to_selection(
    mut commands: Commands,
    windows: Res<Windows>,
    cameras: Query<&Transform, With<Camera>>,
    selections: Query<Entity, With<Selection>>,
) {
    let camera_transform = cameras.single().expect("Wrong amount of cameras.");
    let window = windows.get_primary().unwrap();
    if let Some(pos) = window.cursor_position() {
        let size = Vec2::new(window.width() as f32, window.height() as f32);
        let p = pos - size / 2.0;
        let world_pos = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        let mut pos = Transform::from_xyz(world_pos.x.clone(), world_pos.y.clone(), 0.0);

        // Snap!
        let snapped_pos = (pos.translation / CELL_SIZE).round() * CELL_SIZE;
        pos.translation = snapped_pos;

        let selection = selections.single().expect("Wrong amount of selections.");
        commands.entity(selection).insert(pos);
    }
}

fn click_add(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut map: ResMut<Map>,
    button: Res<Input<MouseButton>>,
    ui_item: Res<UiItem>,
    mode: Res<Mode>,
    selection: Query<&Transform, With<Selection>>,
) {
    if !button.just_pressed(MouseButton::Left) {
        return;
    }
    if *mode != Mode::Add {
        return;
    }

    let transform = selection.single().unwrap();
    add_item(
        &mut commands,
        &mut materials,
        &asset_server,
        &(transform.translation.truncate() / CELL_SIZE),
        &*ui_item,
    );
    let pos = transform.clone().translation.truncate() / CELL_SIZE;
    let pos = GridPosition::new(pos.x.clone() as i32, pos.y.clone() as i32);
    map.items.push(ui_item.into_item(&pos));
}

fn add_item(
    mut commands: &mut Commands,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
    pos: &Vec2,
    ui_item: &UiItem,
) {
    let handle = materials.add(asset_server.load(ui_item.path()).into());
    commands
        .spawn_bundle(SpriteBundle {
            material: handle,
            transform: Transform::from_translation(pos.extend(0.0) * CELL_SIZE),
            ..Default::default()
        })
        .insert(ui_item.clone());
}

fn drag_diff(
    mut last_pos: Local<Vec2>,
    selection: Query<&Transform, With<Selection>>,
    mut drag: ResMut<Drag>,
) {
    let new_pos = selection.single().unwrap();
    let drag_amount = *last_pos - new_pos.translation.truncate();
    *drag = Drag(drag_amount);
    *last_pos = new_pos.translation.truncate();
}

struct Drag(Vec2);

fn drag(
    button: Res<Input<MouseButton>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
    drag: Res<Drag>,
) {
    if !button.pressed(MouseButton::Right) {
        return;
    }

    let mut pos = cameras.single_mut().unwrap();
    let diff = drag.0;
    dbg!(&diff);
    pos.translation += diff.extend(0.0);
}

struct UiFilename(String);

#[derive(PartialEq)]
enum Mode {
    Add,
    Select,
}

#[derive(Serialize, Deserialize)]
struct Map {
    items: Vec<Item>,
}

impl Map {
    fn new() -> Self {
        Self { items: vec![] }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum UiItem {
    Background,
    Warden,
    Prisoner,
    Wall,
    Door,
    Exit,
    Wire,
}

impl UiItem {
    pub fn path(&self) -> PathBuf {
        match self {
            UiItem::Wall => "cells/wall.png".into(),
            UiItem::Door => "cells/door.png".into(),
            UiItem::Exit => "cells/exit.png".into(),
            UiItem::Wire => "cells/wire.png".into(),
            UiItem::Prisoner => "chars/prisoner.png".into(),
            UiItem::Warden => "chars/warden.png".into(),
            _ => todo!(),
        }
    }

    pub fn into_item(self, grid_pos: &GridPosition) -> Item {
        match self {
            UiItem::Wall => Item::Wall(grid_pos.clone()),
            UiItem::Door => Item::Door(grid_pos.clone()),
            UiItem::Exit => Item::Exit(grid_pos.clone()),
            UiItem::Wire => Item::Wire(grid_pos.clone()),
            UiItem::Prisoner => Item::Prisoner(grid_pos.clone()),
            UiItem::Warden => Item::Warden(grid_pos.clone()),
            _ => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Item {
    Background(Background),
    Warden(GridPosition),
    Prisoner(GridPosition),
    Wall(GridPosition),
    Door(GridPosition),
    Exit(GridPosition),
    Wire(GridPosition),
}

impl Item {
    pub fn position(&self) -> Position {
        match self {
            Item::Background(Background { pos, .. }) => pos.clone(),
            Item::Warden(p) => p.to_position(),
            Item::Prisoner(p) => p.to_position(),
            Item::Wall(p) => p.to_position(),
            Item::Door(p) => p.to_position(),
            Item::Exit(p) => p.to_position(),
            Item::Wire(p) => p.to_position(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Background {
    path: String,
    pos: Position,
}
