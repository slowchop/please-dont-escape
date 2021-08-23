use crate::input::exit_on_escape_key;
use crate::map::{update_map_with_walkables, Map, NonWalkable, Walkable};
use crate::path::Path;
use crate::position::{
    apply_velocity, check_velocity_collisions, sync_sprite_positions, Direction, GridPosition,
    Position, Speed, Velocity,
};
use crate::{path, AppState};
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy_egui::egui::FontDefinitions;
use bevy_egui::{egui, EguiContext};
use nalgebra::Vector2;
use rand::{thread_rng, Rng};
use std::ops::{Add, Deref, Sub};

pub const CELL_SIZE: f32 = 32.0;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum Label {
    Setup,
    CheckVelocityCollisions,
    ApplyVelocity,
}

pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Map::new())
            //
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(setup.system().label(Label::Setup)),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(exit_on_escape_key.system())
                    .with_system(ui.system())
                    // This is here because it uses `just_pressed` which will be skipped in the
                    // FixedUpdateStage.
                    .with_system(player_keyboard_action.system()),
            )
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
                    .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
                    .with_system(
                        player_keyboard_movement
                            .system()
                            .before(Label::CheckVelocityCollisions),
                    )
                    .with_system(chase_camera.system())
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
                    .with_system(warden_actions.system())
                    .with_system(prisoner_escape.system()),
            );
    }
}

#[derive(Debug)]
struct KeyboardControl;

#[derive(Debug)]
struct Warden;

#[derive(Debug)]
struct Prisoner;

#[derive(Debug)]
struct ActionRequested;

/// It's the area of a prisoner's room. It is used to know what is outside room room or not.
#[derive(Debug)]
struct PrisonRoom;

#[derive(Debug)]
enum Door {
    Open,
    Closed,
}

#[derive(Debug)]
struct Exit;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut egui_context: ResMut<EguiContext>,
    asset_server: Res<AssetServer>,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.scale = Vec3::new(0.4, 0.4, 1.0);
    commands.spawn_bundle(camera);

    let fonts = FontDefinitions::default();
    egui_context.ctx().set_fonts(fonts);

    let style: egui::Style = egui::Style::default();
    egui_context.ctx().set_style(style);

    let text_map = "\
oWowowxwowowowowowowowowowowowowowowowo o o o o>
o                                w   w        o.
o                         o o o ow  owo o o   o.
o o o o o                 o c c ow  owc p o   o.
o                         o c c sw  swc c o   o.
o   P   o                 o p t.o   o c t.o   o.
o       o                 o o o.o   o o o.o   o.
o d d   o                      .         .    o.
o o o o o x o o o o o o x o o o.o.o.o.o.o.o.o.o.
";

    /*
    P warden player spawn point
    p prisoner spawn point
    o normal wall
    c cell
    s security door
    d office desk
    x exit
    W wire source
    w wire
    > pipe source
    . pipe
    t toilet
     */

    let warden = materials.add(asset_server.load("chars/warden.png").into());
    let prisoner = materials.add(asset_server.load("chars/prisoner.png").into());
    let wall = materials.add(asset_server.load("cells/wall.png").into());
    let exit = materials.add(asset_server.load("cells/exit.png").into());
    let prison_door = materials.add(asset_server.load("cells/prison-door.png").into());

    let mut cell = GridPosition::new(0, 0);
    for line in text_map.split("\n") {
        cell.0.x = 0;
        cell.0.y -= 1;
        for chunk in line.chars().collect::<Vec<_>>().chunks(2) {
            cell.0.x += 1;
            let mut needs_walkable = true;
            let mut needs_cell = false;

            let left = chunk[0];
            let _right = chunk[1];
            match left {
                'P' => {
                    commands
                        .spawn_bundle(sprite(warden.clone(), &cell))
                        .insert(Position::from(&cell))
                        .insert(Direction::new())
                        .insert(Velocity::zero())
                        .insert(Warden)
                        .insert(Speed::good_guy())
                        .insert(KeyboardControl);
                }
                'p' => {
                    commands
                        .spawn_bundle(sprite(prisoner.clone(), &cell))
                        .insert(Position::from(&cell))
                        .insert(Velocity::zero())
                        .insert(Prisoner)
                        .insert(Speed::bad_guy());
                    needs_cell = true;
                }
                'o' => {
                    commands
                        .spawn_bundle(sprite(wall.clone(), &cell))
                        .insert(cell.clone())
                        .insert(NonWalkable);
                    needs_walkable = false;
                }
                'x' => {
                    commands
                        .spawn_bundle(sprite(exit.clone(), &cell))
                        .insert(cell.clone())
                        .insert(Exit);
                }
                's' => {
                    commands
                        .spawn_bundle(sprite(prison_door.clone(), &cell))
                        .insert(cell.clone())
                        .insert(Door::Closed)
                        .insert(NonWalkable);
                    needs_walkable = false;
                }
                'c' => {
                    needs_cell = true;
                }
                _ => {}
            };
            if needs_walkable {
                commands.spawn().insert(cell.clone()).insert(Walkable);
            }
            if needs_cell {
                commands.spawn().insert(cell.clone()).insert(PrisonRoom);
            }
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

fn chase_camera(
    mut camera_query: Query<(&Camera, &mut Transform)>,
    mut player_query: Query<(&KeyboardControl, &Position)>,
) {
    let option_first_camera = camera_query.iter_mut().next();
    let option_first_player = player_query.iter_mut().next();

    if option_first_camera.is_none() {
        return;
    }
    if option_first_player.is_none() {
        return;
    }

    let (_, mut camera_pos) = option_first_camera.unwrap();
    let (_, player_pos) = option_first_player.unwrap();

    camera_pos.translation.x = player_pos.0.x as f32 * CELL_SIZE;
    camera_pos.translation.y = player_pos.0.y as f32 * CELL_SIZE;
}

fn player_keyboard_movement(
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Direction, &Speed), With<KeyboardControl>>,
) {
    for (mut vel, mut dir, speed) in query.iter_mut() {
        let mut new_dir = Direction::default();
        if keys.pressed(KeyCode::A) {
            new_dir.left();
        }
        if keys.pressed(KeyCode::D) {
            new_dir.right();
        }
        if keys.pressed(KeyCode::W) {
            new_dir.up();
        }
        if keys.pressed(KeyCode::S) {
            new_dir.down();
        }
        if new_dir != Direction::default() {
            *dir = new_dir.clone();
        }

        *vel = new_dir.normalized_velocity(speed);
    }
}

fn player_keyboard_action(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut query: Query<Entity, With<KeyboardControl>>,
) {
    for entity in query.iter() {
        if keys.just_pressed(KeyCode::Space) {
            commands.entity(entity).insert(ActionRequested);
        }
    }
}

fn warden_actions(
    mut commands: Commands,
    wardens: Query<(Entity, &Position, &Direction), (With<Warden>, With<ActionRequested>)>,
    mut doors: Query<(Entity, &GridPosition, &mut Door, &mut Visible)>,
) {
    for (warden_ent, warden_pos, warden_dir) in wardens.iter() {
        commands.entity(warden_ent).remove::<ActionRequested>();
        let forward_pos = warden_pos.nearest_cell() + warden_dir;
        for (door_ent, door_grid_pos, mut door, mut visible) in doors.iter_mut() {
            if &forward_pos != door_grid_pos {
                continue;
            }

            match *door {
                Door::Closed => {
                    info!("Door opened");
                    visible.is_visible = false;
                    *door = Door::Open;
                    commands
                        .entity(door_ent)
                        .remove::<NonWalkable>()
                        .insert(Walkable);
                }
                Door::Open => {
                    info!("Door closed");
                    visible.is_visible = true;
                    *door = Door::Closed;
                    commands
                        .entity(door_ent)
                        .remove::<Walkable>()
                        .insert(NonWalkable);
                }
            }
        }
    }
}

fn prisoner_escape(
    mut commands: Commands,
    map: Res<Map>,
    query: Query<(Entity, &Prisoner, &Position), Without<Path>>,
    exits: Query<(&Exit, &GridPosition)>,
) {
    let exit_cell = exits.iter().next();
    if exit_cell.is_none() {
        println!("no exit");
        return;
    }
    let (_, exit_cell) = exit_cell.unwrap();
    for (entity, _prisoner, pos) in query.iter() {
        let found = map.find_path(&pos.nearest_cell(), &exit_cell);
        if let Some((ref steps, _)) = found {
            info!("found path {:?}", found);
            commands.entity(entity).insert(Path::new(steps));
        } else {
            // info!("no path found!");
        }
    }
}
