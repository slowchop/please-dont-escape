use crate::input::exit_on_escape_key;
use crate::map::{Map, NonWalkable, Walkable};
use crate::{map, AppState};
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use nalgebra::Vector2;
use rand::{thread_rng, Rng};
use std::ops::{Add, Deref, Sub};

const CELL_SIZE: f32 = 32.0;

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
                SystemSet::on_update(AppState::MainMenu).with_system(exit_on_escape_key.system()),
            )
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
                    .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
                    .with_system(player_input.system().before(Label::CheckVelocityCollisions))
                    .with_system(
                        move_along_path
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
                    .with_system(map::update_map_with_walkables.system())
                    .with_system(prisoner_escape.system()),
            );
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Cell(pub Vector2<i32>);

impl Cell {
    fn new(x: i32, y: i32) -> Self {
        Self(Vector2::new(x, y))
    }

    pub fn four_directions() -> Vec<Self> {
        vec![
            Cell::new(0, 1),
            Cell::new(0, -1),
            Cell::new(1, 0),
            Cell::new(-1, 0),
        ]
    }
}

impl Add for &Cell {
    type Output = Cell;

    fn add(self, rhs: Self) -> Self::Output {
        let mut c = self.clone();
        c.0 += rhs.0;
        c
    }
}

impl Sub for &Cell {
    type Output = Cell;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut c = self.clone();
        c.0 -= rhs.0;
        c
    }
}

#[derive(Debug, Clone)]
pub struct Position(Vector2<f64>);

impl Position {
    fn new(x: f64, y: f64) -> Self {
        Self(Vector2::new(x, y))
    }

    pub fn nearest_cell(&self) -> Cell {
        Cell::new(self.0.x.round() as i32, self.0.y.round() as i32)
    }

    pub fn to_transform(&self) -> Transform {
        Transform::from_xyz(
            self.0.x.clone() as f32 * CELL_SIZE,
            self.0.y.clone() as f32 * CELL_SIZE,
            0.0,
        )
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

impl Sub for &Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.0 - rhs.0).into()
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.0 - rhs.0).into()
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
struct Path {
    cells: Vec<Cell>,
    current: usize,
}

impl Path {
    fn new(cells: &[Cell]) -> Self {
        Self {
            cells: cells.into(),
            current: 0,
        }
    }

    fn target(&self) -> &Cell {
        &self.cells[self.current]
    }

    fn next(&mut self) -> Option<&Cell> {
        let next_idx = self.current + 1;
        if self.cells.len() == next_idx {
            None
        } else {
            self.current = next_idx;
            Some(self.target())
        }
    }
}

#[derive(Debug)]
struct Speed(f64);

impl Speed {
    fn new(speed: f64) -> Self {
        Self(speed)
    }

    fn good_guy() -> Self {
        Self::new(0.1)
    }

    fn bad_guy() -> Self {
        Self::new(0.04 + thread_rng().gen_range(0.0..0.02))
    }
}

#[derive(Debug)]
struct KeyboardControl;

#[derive(Debug)]
struct Warden;

#[derive(Debug)]
struct Prisoner;

/// It's the area of a prisoner's cell.
#[derive(Debug)]
struct PrisonRoom;

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

    let mut cell = Cell::new(0, 0);
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

fn sprite(material: Handle<ColorMaterial>, cell: &Cell) -> SpriteBundle {
    SpriteBundle {
        material,
        transform: Position::from(cell).to_transform(),
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

fn move_along_path(mut query: Query<(&mut Velocity, &mut Path, &Position, &Speed)>) {
    for (mut vel, mut path, pos, speed) in query.iter_mut() {
        let target: Position = path.target().into();
        let pos: &Position = pos;
        let diff = target - pos.clone();
        let remaining = diff.0.magnitude_squared();

        if remaining < 0.1 {
            let next_target = path.next();
            if next_target.is_none() {
                // TODO: Remove Path
            }
        } else {
            *vel.0 = *(diff.0.normalize() * speed.0);
        }
    }
}

fn check_velocity_collisions(map: Res<Map>, mut query: Query<(&Position, &mut Velocity)>) {
    let map: &Map = map.deref();
    for (pos, mut vel) in query.iter_mut() {
        if !map.is_walkable_pos(&Position::from(pos.0 + vel.0)) {
            // Allow "sliding" on the wall.
            let mut v_vel = vel.clone();
            v_vel.0.x = 0.0;
            let mut h_vel = vel.clone();
            h_vel.0.y = 0.0;
            if map.is_walkable_pos(&Position::from(pos.0 + v_vel.0)) {
                *vel = v_vel;
            } else if map.is_walkable_pos(&Position::from(pos.0 + h_vel.0)) {
                *vel = h_vel;
            } else {
                // Can't slide at all. Probably in a corner.
                // TODO: Maybe trapped inside something that just spawned or closed.
                *vel = Velocity::zero();
            }
        };
    }
}

fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0;
    }
}

fn sync_sprite_positions(mut query: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (pos, mut transform) in query.iter_mut() {
        *transform = pos.to_transform();
    }
}

fn prisoner_escape(
    mut commands: Commands,
    map: Res<Map>,
    query: Query<(Entity, &Prisoner, &Position), Without<Path>>,
    exits: Query<(&Exit, &Cell)>,
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
            info!("no path found!");
        }
    }
}
