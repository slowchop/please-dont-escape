use crate::input::exit_on_escape_key;
use crate::map::{update_map_with_walkables, Item, Map, NonWalkable, PathfindingMap, Walkable};
use crate::path::Path;
use crate::position::{
    apply_velocity, check_velocity_collisions, sync_sprite_positions, Direction, GridPosition,
    Position, Speed, Velocity,
};
use crate::{path, AppState};
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::utils::HashMap;
use bevy_egui::egui::FontDefinitions;
use bevy_egui::{egui, EguiContext};
use nalgebra::Vector2;
use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng, RngCore};
use std::fs::File;
use std::ops::{Add, Deref, Sub};

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
                    .with_system(prisoner_escape.system())
                    //
                    .with_system(damaged_check_if_broken.system())
                    .with_system(damage_wires.system())
                    .with_system(damaged_smoke.system())
                    .with_system(move_smoke.system())
                    .with_system(open_doors_if_any_wires_are_broken.system())
                    // Actions
                    .with_system(warden_actions.system().before(Label::ClearActions))
                    .with_system(clear_actions.system().label(Label::ClearActions)),
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
struct Damaged(Timer);

#[derive(Debug)]
struct Broken;

#[derive(Debug)]
struct Disconnected;

#[derive(Debug, PartialEq)]
enum Action {
    Pending,
    Done,
}

/// It's the area of a prisoner's room. It is used to know what is outside room or not.
#[derive(Debug)]
struct PrisonRoom;

#[derive(Debug)]
struct Escaping;

#[derive(Debug)]
struct SpawnPoint(pub GridPosition);

#[derive(Debug)]
enum Door {
    Open,
    Closed,
}

#[derive(Debug)]
struct Exit;

#[derive(Debug)]
struct Wire;

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

        if needs_cell {
            commands.spawn().insert(cell.clone()).insert(PrisonRoom);
        }
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
            commands.entity(entity).insert(Action::Pending);
        }
    }
}

fn warden_actions(
    mut commands: Commands,
    mut wardens: Query<(&Position, &Direction, &mut Action), With<Warden>>,
    mut doors: Query<(Entity, &GridPosition, &Door)>,
    prisoners: Query<(Entity, &Position, &SpawnPoint), (With<Prisoner>, With<Escaping>)>,
) {
    for (warden_pos, warden_dir, mut action) in wardens.iter_mut() {
        let forward_pos = warden_pos.nearest_cell() + warden_dir;
        for (door_ent, door_grid_pos, door) in doors.iter_mut() {
            if &forward_pos != door_grid_pos {
                continue;
            }
            *action = Action::Done;

            match door {
                Door::Closed => {
                    change_door_state(&mut commands, door_ent, true);
                }
                Door::Open => {
                    change_door_state(&mut commands, door_ent, false);
                }
            }
        }

        if *action == Action::Done {
            continue;
        }

        for (prisoner_ent, prisoner_pos, spawn_point) in prisoners.iter() {
            let dist = warden_pos.distance_to(&prisoner_pos);
            if dist > 1.5 {
                continue;
            }

            info!("prisoner action done!");

            // Temporarily just respawn them!
            let new_pos: Position = spawn_point.0.into();
            commands
                .entity(prisoner_ent)
                .insert(new_pos)
                .insert(Velocity::zero())
                .remove::<Escaping>()
                .remove::<Path>();
        }
    }
}

fn change_door_state(commands: &mut Commands, door_ent: Entity, open: bool) {
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

fn clear_actions(mut commands: Commands, actions: Query<Entity, With<Action>>) {
    for entity in actions.iter() {
        info!("Clearing action for ent {:?}", entity);
        commands.entity(entity).remove::<Action>();
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

fn damage_wires(
    mut commands: Commands,
    good_wires: Query<Entity, (With<Wire>, Without<Damaged>, Without<Broken>)>,
) {
    let mut rng = thread_rng();
    // 1000 seems good
    if rng.next_u32() % 100 != 0 {
        return;
    }

    let ents = good_wires.iter().choose_multiple(&mut rng, 1);
    let ent = ents.get(0);
    info!("damaging: {:?}", ent);
    match ent {
        Some(e) => {
            commands
                .entity(*e)
                .insert(Damaged(Timer::from_seconds(2.0, false)))
                .insert(Smoking(Timer::from_seconds(0.5, true)));
        }
        None => {
            info!("No wires left to smoke");
        }
    };
}

#[derive(Debug)]
struct Smoke;

#[derive(Debug)]
struct Smoking(Timer);

fn damaged_smoke(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut damaged: Query<(&Transform, &mut Smoking)>,
) {
    for (transform, mut timer) in damaged.iter_mut() {
        if !timer.0.tick(time.delta()).just_finished() {
            continue;
        }

        let mut color_material: ColorMaterial = asset_server.load("effects/smoke.png").into();
        let material = materials.add(color_material);
        commands
            .spawn_bundle(SpriteBundle {
                material: material.clone(),
                transform: transform.clone(),
                ..Default::default()
            })
            .insert(Smoke)
            .insert(Alpha(1.0));
    }
}

fn open_doors_if_any_wires_are_broken(
    mut commands: Commands,
    broken_wires: Query<(Entity), (With<Wire>, With<Broken>)>,
    doors: Query<Entity, With<Door>>,
) {
    let mut found = false;
    for _ in broken_wires.iter() {
        found = true;
        break;
    }
    if !found {
        return;
    }

    for door_ent in doors.iter() {
        change_door_state(&mut commands, door_ent, true);
    }
}

fn damaged_check_if_broken(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut damaged: Query<(Entity, &mut Damaged)>,
) {
    for (ent, mut damage) in damaged.iter_mut() {
        if !damage.0.tick(time.delta()).just_finished() {
            continue;
        }

        let mut color_material: ColorMaterial = asset_server.load("cells/wire-broken.png").into();
        let material = materials.add(color_material);
        commands
            .entity(ent)
            .remove::<Damaged>()
            .remove::<Smoking>()
            .insert(Broken)
            .insert(material);
    }
}

struct Alpha(f32);

fn move_smoke(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut smokes: Query<
        (
            Entity,
            &mut Transform,
            &mut Handle<ColorMaterial>,
            &mut Alpha,
        ),
        With<Smoke>,
    >,
) {
    for (ent, mut transform, mut material, mut alpha) in smokes.iter_mut() {
        alpha.0 -= 0.01;
        if alpha.0 <= 0.0 {
            commands.entity(ent).despawn();
            return;
        }

        // none of this works :(
        //
        // let mut color_mat = materials.get_mut(&material).unwrap();
        // color_mat.color = Color::rgba(1.0,0.0,1.0, alpha.0);
        // ColorMaterial::modulated_texture()
        // color_material.color = Color::Rgba {
        //     red: 1.0,
        //     green: 0.0,
        //     blue: 0.0,
        //     alpha: alpha.0,
        // };
        // *material = materials.add(color_material);
        // dbg!("yay");
        transform.translation.y += 0.1;
        // let new_alpha = material.color.a() - 0.001;
        // material.color.set_a(new_alpha);
        // dbg!(material.color);
    }
}
