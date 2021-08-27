use std::fs::File;
use std::ops::{Add, Deref, Sub};

use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::FontDefinitions;
use nalgebra::Vector2;
use rand::{Rng, RngCore, thread_rng};
use rand::prelude::IteratorRandom;

use crate::{AppState, path, player, wires};
use crate::input::exit_on_escape_key;
use crate::map::{Item, Map, NonWalkable, PathfindingMap, update_map_with_walkables, Walkable};
use crate::path::Path;
use crate::position::{
    apply_velocity, check_velocity_collisions, Direction, GridPosition, Position,
    Speed, sync_sprite_positions, Velocity,
};
use crate::wires::{Smoking, Wire};

pub const CELL_SIZE: f32 = 32.0;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum Label {
    Setup,
    CheckVelocityCollisions,
    ApplyVelocity,
    ClearActions,
}

pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(PathfindingMap::new())
            //
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(setup.system().label(Label::Setup)),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(ui.system())
                    // This is here because it uses `just_pressed` which will be skipped in the
                    // FixedUpdateStage.
                    .with_system(player::player_keyboard_action.system()),
            )
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
                    .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
                    .with_system(
                        player::player_keyboard_movement
                            .system()
                            .before(Label::CheckVelocityCollisions),
                    )
                    .with_system(player::chase_camera.system())
                    .with_system(
                        path::move_along_path
                            .system()
                            .before(Label::CheckVelocityCollisions),
                    )
                    .with_system(
                        check_velocity_collisions
                            .system()
                            .label(Label::CheckVelocityCollisions),
                    )
                    .with_system(
                        apply_velocity
                            .system()
                            .after(Label::CheckVelocityCollisions)
                            .label(Label::ApplyVelocity),
                    )
                    .with_system(sync_sprite_positions.system().after(Label::ApplyVelocity))
                    .with_system(update_map_with_walkables.system())
                    .with_system(prisoner_escape.system())
                    //
                    .with_system(wires::damaged_check_if_broken.system())
                    .with_system(wires::damage_wires.system())
                    .with_system(wires::damaged_smoke.system())
                    .with_system(wires::move_smoke.system())
                    .with_system(wires::open_doors_if_any_wires_are_broken.system())
                    // Actions
                    .with_system(player::warden_actions.system().before(Label::ClearActions))
                    .with_system(player::clear_actions.system().label(Label::ClearActions)),
            );
    }
}

#[derive(Debug)]
pub struct KeyboardControl;

#[derive(Debug)]
pub struct Warden;

#[derive(Debug)]
pub struct Prisoner;

// /// It's the area of a prisoner's room. It is used to know what is outside room or not.
// #[derive(Debug)]
// pub struct PrisonRoom;
//
#[derive(Debug)]
pub struct Escaping;

#[derive(Debug)]
pub struct SpawnPoint(pub GridPosition);

#[derive(Debug)]
pub enum Door {
    Open,
    Closed,
}

pub struct Alpha(pub f32);

#[derive(Debug)]
struct Exit;

fn setup(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.scale = Vec3::new(0.4, 0.4, 1.0);
    commands.spawn_bundle(camera);

    let fonts = FontDefinitions::default();
    egui_context.ctx().set_fonts(fonts);

    let style: egui::Style = egui::Style::default();
    egui_context.ctx().set_style(style);

    let warden = materials.add(asset_server.load("chars/warden.png").into());
    let prisoner = materials.add(asset_server.load("chars/prisoner.png").into());
    let wall = materials.add(asset_server.load("cells/wall.png").into());
    let exit = materials.add(asset_server.load("cells/exit.png").into());
    let prison_door = materials.add(asset_server.load("cells/prison-door.png").into());
    let wire = materials.add(asset_server.load("cells/wire.png").into());

    let mut f = File::open("assets/maps/level1.json").expect("Could not open file for reading.");
    let map: Map = serde_json::from_reader(f).expect("Could not read from file.");

    let mut min = GridPosition::zero();
    let mut max = GridPosition::zero();
    let mut first = false;
    let mut items_added: HashMap<GridPosition, ()> = HashMap::default();

    for item_info in &map.items {
        let cell = item_info.pos.nearest_cell_grid_pos();
        let pos: Position = item_info.pos.into();
        if first {
            min = cell.clone();
            max = cell.clone();
            first = false;
        } else {
            if min.0.x > cell.0.x {
                min.0.x = cell.0.x;
            }
            if min.0.y > cell.0.y {
                min.0.y = cell.0.y;
            }
            if max.0.x < cell.0.x {
                max.0.x = cell.0.x;
            }
            if max.0.y < cell.0.y {
                max.0.y = cell.0.y;
            }
        }

        // None, don't affect the map.
        // Some(true) Walkable component
        // Some(false) NonWalkable component
        let mut walkable = None;

        let mut needs_cell = false;

        match &item_info.item {
            Item::Background(bg_path) => {
                let asset: ColorMaterial = asset_server.load(bg_path.as_str()).into();
                let bg = materials.add(asset);
                commands.spawn_bundle(sprite(bg, &cell)).insert(pos);
            }
            Item::Warden => {
                commands
                    .spawn_bundle(sprite(warden.clone(), &cell))
                    .insert(pos)
                    .insert(Direction::new())
                    .insert(Velocity::zero())
                    .insert(Warden)
                    .insert(Speed::good_guy())
                    .insert(KeyboardControl);
            }
            Item::Prisoner => {
                commands
                    .spawn_bundle(sprite(prisoner.clone(), &cell))
                    .insert(Position::from(&cell))
                    .insert(Velocity::zero())
                    .insert(Prisoner)
                    .insert(SpawnPoint(cell.clone()))
                    .insert(Speed::bad_guy());
                needs_cell = true;
            }
            Item::Wall => {
                commands
                    .spawn_bundle(sprite(wall.clone(), &cell))
                    .insert(cell.clone());
                walkable = Some(false);
            }
            Item::Door => {
                commands
                    .spawn_bundle(sprite(prison_door.clone(), &cell))
                    .insert(cell.clone())
                    .insert(Door::Closed);
                walkable = Some(false);
            }
            Item::Exit => {
                commands
                    .spawn_bundle(sprite(exit.clone(), &cell))
                    .insert(cell.clone())
                    .insert(Exit);
            }
            Item::Wire => {
                commands
                    .spawn_bundle(sprite(wire.clone(), &cell))
                    .insert(cell.clone())
                    .insert(Wire);
            }
        };

        if walkable == Some(false) {
            items_added.insert(cell, ());
            commands.spawn().insert(cell.clone()).insert(NonWalkable);
        }

        // if needs_cell {
        //     commands.spawn().insert(cell.clone()).insert(PrisonRoom);
        // }
    }

    for x in min.0.x..max.0.x {
        for y in min.0.y..max.0.y {
            let cell = GridPosition::new(x, y);
            if items_added.contains_key(&cell) {
                continue;
            }

            commands.spawn().insert(cell).insert(Walkable);
        }
    }
}

fn ui(
    egui_context: ResMut<EguiContext>,
    wardens: Query<(&Position, &Direction), With<Warden>>,
    prisoners: Query<&Position, With<Prisoner>>,
) {
    egui::Window::new("Debug").show(egui_context.ctx(), |ui| {
        for (pos, dir) in wardens.iter() {
            ui.heading("Warden");
            ui.label(format!("{:?}", dir));
            ui.label(format!("{:?}", pos));
        }

        for pos in prisoners.iter() {
            ui.heading("Prisoner");
            ui.label(format!("{:?}", pos));
        }
    });
}

fn sprite(material: Handle<ColorMaterial>, cell: &GridPosition) -> SpriteBundle {
    SpriteBundle {
        material,
        transform: Position::from(cell).to_transform(),
        ..Default::default()
    }
}

pub fn change_door_state(commands: &mut Commands, door_ent: Entity, open: bool) {
    let mut e = commands.entity(door_ent);

    let door = match open {
        true => Door::Open,
        false => Door::Closed,
    };

    e //
        .insert(door)
        .remove::<Walkable>()
        .remove::<NonWalkable>()
        .insert(Visible {
            is_visible: !open,
            is_transparent: false,
        });

    if open {
        e.insert(Walkable);
    } else {
        e.insert(NonWalkable);
    }
}

fn prisoner_escape(
    mut commands: Commands,
    map: Res<PathfindingMap>,
    query: Query<(Entity, &Prisoner, &Position), Without<Path>>,
    exits: Query<(&Exit, &GridPosition)>,
) {
    let mut rng = thread_rng();
    let exit_cells = exits.iter().choose_multiple(&mut rng, 1);
    let exit_cell = exit_cells.get(0);
    if exit_cell.is_none() {
        // warn!("No exits found!");
        return;
    }
    let (_, exit_cell) = exit_cell.unwrap();
    for (entity, _prisoner, pos) in query.iter() {
        let found = map.find_path(&pos.nearest_cell(), &exit_cell);
        if let Some((ref steps, _)) = found {
            // info!("found path {:?}", found);
            commands
                .entity(entity)
                .insert(Path::new(steps))
                .insert(Escaping);
        } else {
            // info!("no path found!");
        }
    }
}

