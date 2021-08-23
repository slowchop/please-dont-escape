use crate::input::exit_on_escape_key;
use crate::AppState;
use bevy::core::{FixedTimestep, FixedTimesteps};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

const CELL_SIZE: f32 = 32.0;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup.system()))
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu).with_system(exit_on_escape_key.system()),
            )
            // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
                    .with_system(sync_sprite_positions.system())
                    .with_system(player_input.system()),
            );
    }
}

#[derive(Debug, Clone)]
struct Cell {
    x: i32,
    y: i32,
}

impl Cell {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
struct Position {
    x: f64,
    y: f64,
}

impl Position {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<&Cell> for Position {
    fn from(cell: &Cell) -> Self {
        Position::new(cell.clone().x as f64, cell.clone().y as f64)
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

#[derive(Debug)]
struct Map {
    cells: bevy::utils::HashMap<Cell, ()>,
    size: Cell,
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
struct Person;

#[derive(Debug)]
struct Player;

#[derive(Debug)]
struct KeyboardControl;

#[derive(Debug)]
struct Warden;

#[derive(Debug)]
struct Prisoner;

#[derive(Debug)]
struct Wall;

#[derive(Debug)]
struct Exit;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // commands
    //     .spawn()
    //     .insert(Player)
    //     .insert(Person)
    //     .insert(Position::new(2, 2))
    //     .insert(Speed::new(1.0));
    //
    let map = "\
oWowowxwowowowowowowowowowowowowowowowo o o o o>
o                                w   w        o.
o                         j j j jw  jwj j j   o.
o o o o o                 j     jw  jw  p j   o.
o                         j     sw  sw    j   o.
o   W   o                 j p t.j   j   t.j   o.
o       o                 j j j.j   j j j.j   o.
o d d   o                      .         .    o.
o o o o o x o o o o o o x o o o.o.o.o.o.o.o.o.o.
";

    /*
    W warden player spawn point
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
    let wall = materials.add(asset_server.load("cells/wall.png").into());
    let exit = materials.add(asset_server.load("cells/exit.png").into());

    let mut cell = Cell::new(0, 0);
    for line in map.split("\n") {
        cell.x = 0;
        cell.y -= 1;
        for chunk in line.chars().collect::<Vec<_>>().chunks(2) {
            cell.x += 1;

            let left = chunk[0];
            let right = chunk[1];
            match left {
                'W' => {
                    commands
                        .spawn_bundle(sprite(warden.clone(), &cell))
                        .insert(Position::from(&cell))
                        .insert(Person)
                        .insert(Warden)
                        .insert(Player)
                        .insert(Speed::person())
                        .insert(KeyboardControl);
                }
                'o' => {
                    commands
                        .spawn_bundle(sprite(wall.clone(), &cell))
                        .insert(cell.clone())
                        .insert(Wall);
                }
                'x' => {
                    commands
                        .spawn_bundle(sprite(exit.clone(), &cell))
                        .insert(cell.clone())
                        .insert(Exit);
                }
                _ => {}
            };
        }
    }
}

fn sprite(material: Handle<ColorMaterial>, pos: &Cell) -> SpriteBundle {
    SpriteBundle {
        material,
        transform: Transform::from_xyz(
            pos.x.clone() as f32 * CELL_SIZE,
            pos.y.clone() as f32 * CELL_SIZE,
            0.0,
        ),
        ..Default::default()
    }
}

fn player_input(
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&KeyboardControl, &mut Position, &Speed)>,
) {
    for (_, mut pos, speed) in query.iter_mut() {
        if keys.pressed(KeyCode::A) {
            pos.x -= speed.0;
        }
        if keys.pressed(KeyCode::D) {
            pos.x += speed.0;
        }
        if keys.pressed(KeyCode::W) {
            pos.y += speed.0;
        }
        if keys.pressed(KeyCode::S) {
            pos.y -= speed.0;
        }
    }
}

fn sync_sprite_positions(mut query: Query<(&Position, &mut Transform), (Changed<Position>)>) {
    for (pos, mut transform) in query.iter_mut() {
        *transform = Transform::from_xyz(pos.x.clone() as f32 * CELL_SIZE, pos.y.clone() as f32 * CELL_SIZE, 0.0);
    }
}

fn move_things(mut query: Query<(&mut Position, &Speed)>) {
    // for (pos, speed) in query.iter_mut() {
    //     info!("{:?}", cell);
    // }
}
