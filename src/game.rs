use std::fs::File;
use std::ops::{Add, Deref, Sub};

use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::utils::HashMap;
use bevy_egui::egui::FontDefinitions;
use bevy_egui::{egui, EguiContext};
use nalgebra::Vector2;
use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng, RngCore};

use crate::input::exit_on_escape_key;
use crate::map::{Item, ItemInfo, Map, PathfindingMap};
use crate::path::Path;
use crate::position::{
    apply_velocity, check_velocity_collisions, sync_sprite_positions, Direction, GridPosition,
    Position, Speed, Velocity,
};
use crate::wires::{Smoking, Wire};
use crate::{path, player, wires, AppState};

pub const GRID_SIZE: f32 = 160.0;

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
pub struct Door(pub bool);

pub struct Alpha(pub f32);

#[derive(Debug)]
struct Exit;

fn setup(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut pathfinding_map: ResMut<PathfindingMap>,
    asset_server: Res<AssetServer>,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.scale = Vec3::new(2.0, 2.0, 1.0);
    commands.spawn_bundle(camera);

    let fonts = FontDefinitions::default();
    egui_context.ctx().set_fonts(fonts);

    let style: egui::Style = egui::Style::default();
    egui_context.ctx().set_style(style);

    let mut f = File::open("assets/maps/level1.json").expect("Could not open file for reading.");
    let map: Map = serde_json::from_reader(f).expect("Could not read from file.");

    let mut min = GridPosition::zero();
    let mut max = GridPosition::zero();
    let mut first = false;
    let mut items_added: HashMap<GridPosition, ()> = HashMap::default();

    for item_info in &map.items {
        let grid_pos = item_info.position.nearest_cell_grid_pos();
        let pos: Position = item_info.position.into();
        if first {
            min = grid_pos.clone();
            max = grid_pos.clone();
            first = false;
        } else {
            if min.0.x > grid_pos.0.x {
                min.0.x = grid_pos.0.x;
            }
            if min.0.y > grid_pos.0.y {
                min.0.y = grid_pos.0.y;
            }
            if max.0.x < grid_pos.0.x {
                max.0.x = grid_pos.0.x;
            }
            if max.0.y < grid_pos.0.y {
                max.0.y = grid_pos.0.y;
            }
        }

        let mut walkable = true;

        let handle = materials.add(asset_server.load(item_info.item.path()).into());
        let mut ent = commands.spawn_bundle(sprite(handle, &grid_pos));
        ent.insert(pos).insert(item_info.clone());

        match &item_info.item {
            Item::Background(_) => {
                ent.insert(grid_pos);
            }
            Item::Warden => {
                ent //
                    .insert(pos)
                    .insert(Direction::new())
                    .insert(Velocity::zero())
                    .insert(Warden)
                    .insert(Speed::good_guy())
                    .insert(KeyboardControl);
            }
            Item::Prisoner => {
                ent //
                    .insert(pos)
                    .insert(Velocity::zero())
                    .insert(Prisoner)
                    .insert(SpawnPoint(grid_pos.clone()))
                    .insert(Speed::bad_guy());
            }
            Item::Wall => {
                ent.insert(grid_pos);
                walkable = false;
            }
            Item::WallCorner => {
                ent.insert(grid_pos);
                walkable = false;
            }
            Item::Door => {
                ent.insert(grid_pos).insert(Door(false));
                walkable = false;
            }
            Item::Exit => {
                ent.insert(grid_pos).insert(Exit);
            }
            Item::Wire => {
                ent.insert(grid_pos).insert(Wire);
            }
            Item::GeneralTile => {
                ent.insert(grid_pos);
            }
            Item::CellTile => {
                ent.insert(grid_pos);
            }
        };

        // Fill out the "shape" of the item
        for delta in &item_info.shape().0 {
            let delta_cell = &grid_pos + delta;
            if walkable == false {
                items_added.insert(delta_cell, ());
                pathfinding_map.walkable_cells.insert(delta_cell, false);
            }
        }
    }

    for x in min.0.x..max.0.x {
        for y in min.0.y..max.0.y {
            let cell = GridPosition::new(x, y);
            if items_added.contains_key(&cell) {
                continue;
            }
            pathfinding_map.walkable_cells.insert(cell.clone(), true);
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

pub fn change_door_state(
    commands: &mut Commands,
    pathfinding_map: &mut PathfindingMap,
    door_ent: Entity,
    door_grid_pos: &GridPosition,
    door_item_info: &ItemInfo,
    open: bool,
) {
    let mut e = commands.entity(door_ent);

    dbg!(&open);

    e //
        .insert(Door(open))
        .insert(Visible {
            is_visible: !open,
            is_transparent: false,
        });

    for delta in door_item_info.shape().0 {
        let delta_cell = door_grid_pos + &delta;
        pathfinding_map.walkable_cells.insert(delta_cell, open);
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
