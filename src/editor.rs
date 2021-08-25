use crate::position::{GridPosition, Position};
use crate::AppState;
use bevy::prelude::*;
use bevy::utils::StableHashMap;
use bevy_egui::egui::{FontDefinitions, Ui};
use bevy_egui::{egui, EguiContext};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;

pub struct Editor;

impl Plugin for Editor {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            .insert_resource(Map::new())
            .insert_resource(UiFilename("level1".into()))
            .insert_resource(Mode::Add)
            .insert_resource(UiItem::Wall)
            //
            .add_system_set(SystemSet::on_enter(AppState::Editor).with_system(setup.system()))
            .add_system_set(
                SystemSet::on_update(AppState::Editor).with_system(ui.system()), // .with_system(player_keyboard_action.system()),
            );
    }
}

fn setup(mut egui_context: ResMut<EguiContext>) {
    let fonts = FontDefinitions::default();
    egui_context.ctx().set_fonts(fonts);

    let style: egui::Style = egui::Style::default();
    egui_context.ctx().set_style(style);
}

fn ui(
    egui_context: ResMut<EguiContext>,
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
                if ui.button("Load").clicked() {};
                if ui.button("Save").clicked() {
                    let serialized = serde_json::to_vec(&*map).unwrap();
                    let path = PathBuf::from(format!("assets/maps/{}.json", &ui_filename.0));
                    info!("Saving to {:?}", &path);
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

#[derive(PartialEq)]
enum UiItem {
    Background,
    Warden,
    Prisoner,
    Wall,
    Door,
    Exit,
    Wire,
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

#[derive(Serialize, Deserialize)]
struct Background {
    path: String,
    pos: Position,
}
