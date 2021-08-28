use crate::game;
use crate::game::{Door, Escaping, KeyboardControl, Prisoner, SpawnPoint, Warden, GRID_SIZE};
use crate::map::{ItemInfo, PathfindingMap};
use crate::path::Path;
use crate::position::{Direction, GridPosition, Position, Speed, Velocity};
use crate::wires::{Broken, Damaged, Smoking, Wire};
use bevy::prelude::*;
use bevy::render::camera::Camera;

#[derive(Debug, PartialEq)]
pub enum Action {
    Pending,
    Done,
}

pub fn chase_camera(
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

    camera_pos.translation.x = player_pos.0.x as f32 * GRID_SIZE;
    camera_pos.translation.y = player_pos.0.y as f32 * GRID_SIZE;
}

pub fn player_keyboard_movement(
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

pub fn player_keyboard_action(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut query: Query<Entity, With<KeyboardControl>>,
) {
    for entity in query.iter() {
        if keys.just_pressed(KeyCode::Space) {
            commands.entity(entity).insert(Action::Pending);
        }
    }
}

pub fn warden_actions(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut pathfinding_map: ResMut<PathfindingMap>,
    mut wardens: Query<(&Position, &Direction, &mut Action), With<Warden>>,
    mut doors: Query<(Entity, &GridPosition, &Door, &ItemInfo)>,
    prisoners: Query<(Entity, &Position, &SpawnPoint), (With<Prisoner>, With<Escaping>)>,
    broken_wires: Query<(Entity, &GridPosition, Option<&Broken>, Option<&Damaged>), With<Wire>>,
) {
    for (warden_pos, warden_dir, mut action) in wardens.iter_mut() {
        let forward_pos = &warden_pos.nearest_cell() + warden_dir;
        for (door_ent, door_grid_pos, door, door_item_info) in doors.iter_mut() {
            let mut colliding = false;
            for delta in door_item_info.shape().0 {
                let delta_pos = door_grid_pos + &delta;
                if forward_pos == delta_pos {
                    colliding = true;
                    break;
                }
            }
            if !colliding {
                continue;
            }

            *action = Action::Done;

            game::change_door_state(
                &mut commands,
                &mut pathfinding_map,
                door_ent,
                door_grid_pos,
                door_item_info,
                !door.0,
            );
        }

        if *action == Action::Done {
            continue;
        }

        for (prisoner_ent, prisoner_pos, spawn_point) in prisoners.iter() {
            let dist = warden_pos.distance_to(&prisoner_pos);
            if dist > 1.5 {
                continue;
            }

            // Temporarily just respawn them!
            let new_pos: Position = spawn_point.0.into();
            commands
                .entity(prisoner_ent)
                .insert(new_pos)
                .insert(Velocity::zero())
                .remove::<Escaping>()
                .remove::<Path>();
        }

        for (wire_ent, wire_pos, maybe_broken, maybe_damaged) in broken_wires.iter() {
            if maybe_broken.is_none() && maybe_damaged.is_none() {
                continue;
            }
            let dist = warden_pos.distance_to(&wire_pos.into());
            if dist > 1.5 {
                continue;
            }

            let wire = materials.add(asset_server.load("cells/wire.png").into());
            commands
                .entity(wire_ent)
                .remove::<Smoking>()
                .remove::<Broken>()
                .remove::<Damaged>()
                .insert(wire);
        }
    }
}

pub fn clear_actions(mut commands: Commands, actions: Query<Entity, With<Action>>) {
    for entity in actions.iter() {
        info!("Clearing action for ent {:?}", entity);
        commands.entity(entity).remove::<Action>();
    }
}
