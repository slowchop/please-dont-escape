use crate::game::CELL_SIZE;
use crate::position::{FlexPosition, GridPosition, Position};
use crate::AppState;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::utils::StableHashMap;
use bevy_egui::egui::{Align2, FontDefinitions, Ui};
use bevy_egui::{egui, EguiContext};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::fs::File;
use std::io::Write;
use std::ops::{Add, Deref};
use std::path::PathBuf;
use crate::map::{Map, Item, ItemInfo};

pub struct Editor;

impl Plugin for Editor {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            .insert_resource(Drag(Vec2::default()))
            .insert_resource(Map::new())
            .insert_resource(UiFilename("level1".into()))
            .insert_resource(Mode::Add)
            .insert_resource(Item::Wall)
            .insert_resource(SelectedItem::Nothing)
            //
            .add_system_set(SystemSet::on_enter(AppState::Editor).with_system(setup.system()))
            .add_system_set(
                SystemSet::on_update(AppState::Editor)
                    .with_system(ui.system())
                    .with_system(camera_to_selection.system())
                    .with_system(click_add.system())
                    .with_system(click_select.system())
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

fn clear_map(
    mut commands: &mut Commands,
    items: &Query<(Entity, &ItemInfo)>,
){
    for (ent, _) in items.iter() {
        commands.entity(ent).despawn();
    }
}

fn ui(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut ui_filename: ResMut<UiFilename>,
    mut mode: ResMut<Mode>,
    mut item: ResMut<Item>,
    mut map: ResMut<Map>,
    selected_item: Res<SelectedItem>,
    items: Query<(Entity, &ItemInfo)>,
) {
    egui::Window::new("Editor")
        .default_width(200.0)
        .show(egui_context.ctx(), |ui| {
            if ui.button("New").clicked() {
                clear_map(&mut commands, &items);
            };

            ui.horizontal(|ui| {
                ui.label("Filename:");
                ui.text_edit_singleline(&mut ui_filename.0);
            });

            ui.horizontal(|ui| {
                let path = PathBuf::from(format!("assets/maps/{}.json", &ui_filename.0));
                if ui.button("Load").clicked() {
                    info!("Loading from {:?}", &path);
                    clear_map(&mut commands, &items);
                    let mut f = File::open(&path).expect("Could not open file for reading.");
                    *map = serde_json::from_reader(f).expect("Could not read from file.");

                    for item in &map.items {
                        add_item(&mut commands, &mut materials, &asset_server, &*item);
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
                select_mode(ui, "Select Specific", &mut mode, Mode::SelectSpecific);
            });
            ui.heading("Item");
            ui.horizontal_wrapped(|ui| {
                select_item(ui, "Wall", &mut item, Item::Wall);
                select_item(ui, "Warden Spawn", &mut item, Item::Warden);
                select_item(ui, "Prisoner Spawn", &mut item, Item::Prisoner);
                select_item(ui, "Security Door", &mut item, Item::Door);
                select_item(ui, "Exit", &mut item, Item::Exit);
                select_item(ui, "Wire", &mut item, Item::Wire);
                select_item(ui, "Background Image", &mut item, Item::Background("menus/logo.png".into()));

                if let Item::Background(b) = &mut *item {
                    ui.horizontal(|ui| {
                        ui.label("Image path:");
                        ui.text_edit_singleline(b);
                    });
                }
            });

            ui.separator();

            ui.heading("Selected");
            match &*selected_item {
                SelectedItem::Nothing => {
                    ui.label("Nothing selected");
                }
                SelectedItem::Item(selected_item_info) => {
                    ui.label(format!("{:#?}", selected_item_info));
                    if ui.button("Delete").clicked() {

                        // Remove offending item from map.
                        map.items.retain(|i| i != selected_item_info);

                        // Remove from scene.
                        for (entity, item_info) in items.iter() {
                            if item_info != selected_item_info {
                                continue;
                            }

                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        });
}

// TODO: Work out how to make generic
fn select_item(ui: &mut Ui, title: &str, item: &mut ResMut<Item>, new_item: Item) {
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
    mode: Res<Mode>,
    item: Res<Item>,
    selection: Query<&Transform, With<Selection>>,
    egui_context: Res<EguiContext>
) {
    if egui_context.ctx().is_pointer_over_area() {
        return;
    }
    if !button.just_pressed(MouseButton::Left) {
        return;
    }
    if *mode != Mode::Add {
        return;
    }

    let transform = selection.single().unwrap();
    let pos: Position = (transform.translation.truncate() / CELL_SIZE).into();
    let item_info = ItemInfo {
        item: item.clone(),
        pos: FlexPosition::Grid(pos.nearest_cell()),
    };

    add_item(&mut commands, &mut materials, &asset_server, &item_info);
    map.items.push(item_info);
}

#[derive(Debug)]
enum SelectedItem {
    Nothing,
    Item(ItemInfo),
}

fn click_select(
    button: Res<Input<MouseButton>>,
    map: Res<Map>,
    selection: Query<&Transform, With<Selection>>,
    mode: Res<Mode>,
    item: Res<Item>,
    mut selected_item: ResMut<SelectedItem>,
    egui_context: Res<EguiContext>
) {
    if egui_context.ctx().is_pointer_over_area() {
        return;
    }
    if !button.just_pressed(MouseButton::Left) {
        return;
    }
    if *mode != Mode::Select && *mode != Mode::SelectSpecific {
        return;
    }

    let selection_pos: Position =
        (selection.single().unwrap().translation.truncate() / CELL_SIZE).into();

    for scan_item_info in &map.items {
        let scan_pos: Position = scan_item_info.pos.into();
        if selection_pos.distance_to(&scan_pos) < 0.5 {
            if *mode == Mode::Select {
                *selected_item = SelectedItem::Item(scan_item_info.clone());
                return;
            } else {
                if scan_item_info.item == *item {
                    *selected_item = SelectedItem::Item(scan_item_info.clone());
                    return;
                }
            }
        }
    }

    *selected_item = SelectedItem::Nothing;
}

fn add_item(
    mut commands: &mut Commands,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
    item_info: &ItemInfo,
) {
    let handle = materials.add(asset_server.load(item_info.item.path()).into());
    let pos: Position = item_info.pos.into();
    commands
        .spawn_bundle(SpriteBundle {
            material: handle,
            transform: Transform::from_translation((pos * CELL_SIZE as f64).into()),
            ..Default::default()
        })
        .insert(item_info.clone());
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
    SelectSpecific,
}

