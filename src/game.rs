use crate::input::exit_on_escape_key;
use crate::AppState;
use bevy::core::{FixedTimestep, FixedTimesteps};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use nalgebra::Vector2;
use std::collections::HashMap;
use pathfinding::prelude::astar;

const CELL_SIZE: f32 = 32.0;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum Label {
    Setup,
    PlayerInput,
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
                SystemSet::on_update(AppState::MainMenu).with_system(exit_on_escape_key.system()),
            )
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
                    .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
                    .with_system(player_input.system().label(Label::PlayerInput))
                    .with_system(
                        check_velocity_collisions
                            .system()
                            .after(Label::PlayerInput)
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
                    .with_system(prisoner_escape.system()),
            );
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Cell(Vector2<i32>);

impl Cell {
    fn new(x: i32, y: i32) -> Self {
        Self(Vector2::new(x, y))
    }
}

#[derive(Debug, Clone)]
struct Position(Vector2<f64>);

impl Position {
    fn new(x: f64, y: f64) -> Self {
        Self(Vector2::new(x, y))
    }

    fn nearest_cell(&self) -> Cell {
        Cell::new(self.0.x.round() as i32, self.0.y.round() as i32)
    }
}

impl From<&Cell> for Position {
    fn from(cell: &Cell) -> Self {
        Position::new(cell.clone().0.x as f64, cell.clone().0.y as f64)
    }
}

impl From<Vector2<f64>> for Position {
    fn from(v: Vector2<f64>) -> Self {
        Self(v)
    }
}

#[derive(Debug, Clone)]
struct Velocity(Vector2<f64>);

impl Velocity {
    fn new(x: f64, y: f64) -> Self {
        Self(Vector2::new(x, y))
    }

    fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

#[derive(Debug)]
struct Path(Vec<Cell>);

#[derive(Debug)]
struct Speed(f64);

impl Speed {
    fn new(speed: f64) -> Self {
        Self(speed)
    }

    fn person() -> Self {
        Self::new(0.1)
    }
}

/// Specific cells that can be walked on. This should be added when NonWalkable was removed.
#[derive(Debug)]
struct Walkable;

#[derive(Debug)]
struct NonWalkable;

#[derive(Debug)]
struct Map {
    walkable_cells: bevy::utils::HashMap<Cell, bool>,
}

impl Map {
    fn new() -> Self {
        Self {
            walkable_cells: bevy::utils::HashMap::default(),
        }
    }
}

#[derive(Debug)]
struct MapGraph {
    nodes: Vec<Cell>,
}

#[derive(Debug)]
struct MapNode {
    cost: u8,
    neighbours: Vec<Cell>,
}

#[derive(Debug)]
struct KeyboardControl;

#[derive(Debug)]
struct Warden;

#[derive(Debug)]
struct Prisoner;

#[derive(Debug)]
struct Exit;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let text_map = "\
oWowowxwowowowowowowowowowowowowowowowo o o o o>
o                                w   w        o.
o                         o o o ow  owo o o   o.
o o o o o                 o     ow  ow  p o   o.
o                         o     sw  sw    o   o.
o   P   o                 o p t.o   o   t.o   o.
o       o                 o o o.o   o o o.o   o.
o d d   o                      .         .    o.
o o o o o x o o o o o o x o o o.o.o.o.o.o.o.o.o.
";

    /*
    P warden player spawn point
    p prisoner spawn point
    o normal wall
    j jail wall
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

    let mut cell = Cell::new(0, 0);
    for line in text_map.split("\n") {
        cell.0.x = 0;
        cell.0.y -= 1;
        for chunk in line.chars().collect::<Vec<_>>().chunks(2) {
            cell.0.x += 1;
            let mut needs_walkable = true;

            let left = chunk[0];
            let right = chunk[1];
            match left {
                'P' => {
                    commands
                        .spawn_bundle(sprite(warden.clone(), &cell))
                        .insert(Position::from(&cell))
                        .insert(Velocity::zero())
                        .insert(Warden)
                        .insert(Speed::person())
                        .insert(KeyboardControl);
                }
                'p' => {
                    commands
                        .spawn_bundle(sprite(prisoner.clone(), &cell))
                        .insert(Position::from(&cell))
                        .insert(Velocity::zero())
                        .insert(Prisoner)
                        .insert(Speed::person());
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
                _ => {}
            };
            if needs_walkable {
                commands.spawn().insert(cell.clone()).insert(Walkable);
            }
        }
    }
}

fn sprite(material: Handle<ColorMaterial>, pos: &Cell) -> SpriteBundle {
    SpriteBundle {
        material,
        transform: Transform::from_xyz(
            pos.0.x.clone() as f32 * CELL_SIZE,
            pos.0.y.clone() as f32 * CELL_SIZE,
            0.0,
        ),
        ..Default::default()
    }
}

fn player_input(
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&KeyboardControl, &mut Velocity, &Speed)>,
) {
    for (_, mut vel, speed) in query.iter_mut() {
        vel.0.x = 0.0;
        vel.0.y = 0.0;

        if keys.pressed(KeyCode::A) {
            vel.0.x -= 1.0;
        }
        if keys.pressed(KeyCode::D) {
            vel.0.x += 1.0;
        }
        if keys.pressed(KeyCode::W) {
            vel.0.y += 1.0;
        }
        if keys.pressed(KeyCode::S) {
            vel.0.y -= 1.0;
        }

        if vel.0.magnitude() > 0.0 {
            vel.0 = vel.0.normalize();
        }
        vel.0 *= speed.0;
    }
}

fn check_velocity_collisions(map: Res<Map>, mut query: Query<(&Position, &mut Velocity)>) {
    for (pos, mut vel) in query.iter_mut() {
        if !is_walkable(&map, &Position::from(pos.0 + vel.0)) {
            // Allow "sliding" on the wall.
            let mut v_vel = vel.clone();
            v_vel.0.x = 0.0;
            let mut h_vel = vel.clone();
            h_vel.0.y = 0.0;
            if is_walkable(&map, &Position::from(pos.0 + v_vel.0)) {
                *vel = v_vel;
            } else if is_walkable(&map, &Position::from(pos.0 + h_vel.0)) {
                *vel = h_vel;
            } else {
                // Can't slide at all. Probably in a corner.
                // TODO: Maybe trapped inside something that just spawned or closed.
                *vel = Velocity::zero();
            }
        };
    }
}

fn is_walkable(map: &Map, pos: &Position) -> bool {
    let cell = pos.nearest_cell();
    *map.walkable_cells.get(&cell).unwrap_or(&false)
}

fn update_map_with_walkables(
    mut map: ResMut<Map>,
    query: Query<
        (&Cell, Option<&NonWalkable>, Option<&Walkable>),
        (Or<(Added<NonWalkable>, Added<Walkable>)>),
    >,
) {
    for (cell, non_walk, walk) in query.iter() {
        let walkable = match (non_walk.is_some(), walk.is_some()) {
            (false, true) => true,
            (true, false) => false,
            _ => panic!("Something funny is going on with walk vs non walk."),
        };
        map.walkable_cells.insert(cell.clone(), walkable);
    }
}

fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0;
    }
}

fn sync_sprite_positions(mut query: Query<(&Position, &mut Transform), (Changed<Position>)>) {
    for (pos, mut transform) in query.iter_mut() {
        *transform = Transform::from_xyz(
            pos.0.x.clone() as f32 * CELL_SIZE,
            pos.0.y.clone() as f32 * CELL_SIZE,
            0.0,
        );
    }
}

fn prisoner_escape(mut commands: Commands, query: Query<(Entity, &Prisoner, &Position), Without<Path>>, exits: Query<(&Exit, &Cell)>) {
    let exit = exits.iter().next().unwrap();
    for (entity, prisoner, pos) in query.iter() {
        let cell = pos.nearest_cell();
        // astar(cell, )
        info!("entity {:?}", entity);
        commands.entity(entity).insert(Path(vec![]));
    }
}
