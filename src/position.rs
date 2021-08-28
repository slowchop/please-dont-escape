use crate::game::GRID_SIZE;
use crate::map::PathfindingMap;
use bevy::prelude::*;
use core::convert::From;
use nalgebra::Vector2;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Deref, Div, Sub, Mul};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GridPosition(pub Vector2<i32>);

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self(Vector2::new(x, y))
    }

    pub fn four_directions() -> Vec<Self> {
        vec![
            GridPosition::new(0, 1),
            GridPosition::new(0, -1),
            GridPosition::new(1, 0),
            GridPosition::new(-1, 0),
        ]
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn to_position(&self) -> Position {
        Position::new(self.0.x as f64, self.0.y as f64)
    }
}

impl Add for &GridPosition {
    type Output = GridPosition;

    fn add(self, rhs: Self) -> Self::Output {
        let mut c = self.clone();
        c.0 += rhs.0;
        c
    }
}

impl Sub for &GridPosition {
    type Output = GridPosition;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut c = self.clone();
        c.0 -= rhs.0;
        c
    }
}

impl Add<&Direction> for GridPosition {
    type Output = GridPosition;

    fn add(self, rhs: &Direction) -> Self::Output {
        let mut pos = self.clone();
        let rhs = rhs.to_grid_pos();
        pos.0.x += rhs.0.x;
        pos.0.y += rhs.0.y;
        pos
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FlexPosition {
    Position(Position),
    Grid(GridPosition),
}

impl FlexPosition {
   pub fn nearest_cell_grid_pos(&self) -> GridPosition {
       match self {
           FlexPosition::Position(p) => p.nearest_cell(),
           FlexPosition::Grid(g) => g.clone(),
       }
   }
}

impl Into<Position> for FlexPosition {
    fn into(self) -> Position {
        match self {
            FlexPosition::Position(p) => p,
            FlexPosition::Grid(g) => g.to_position(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position(pub Vector2<f64>);

impl Position {
    fn new(x: f64, y: f64) -> Self {
        Self(Vector2::new(x, y))
    }

    pub fn nearest_cell(&self) -> GridPosition {
        GridPosition::new(self.0.x.round() as i32, self.0.y.round() as i32)
    }

    pub fn distance_to(&self, other: &Position) -> f64 {
        (self.0 - other.0).magnitude()
    }

    pub fn to_transform(&self) -> Transform {
        Transform::from_xyz(
            self.0.x.clone() as f32 * GRID_SIZE,
            self.0.y.clone() as f32 * GRID_SIZE,
            0.0,
        )
    }
}

impl From<&GridPosition> for Position {
    fn from(cell: &GridPosition) -> Self {
        Position::new(cell.clone().0.x as f64, cell.clone().0.y as f64)
    }
}

impl From<GridPosition> for Position {
    fn from(cell: GridPosition) -> Self {
        // Second clone not required since we're consuming it.
        Position::new(cell.clone().0.x as f64, cell.0.y as f64)
    }
}

impl From<Vector2<f64>> for Position {
    fn from(v: Vector2<f64>) -> Self {
        Self(v)
    }
}

impl Into<Vec2> for Position {
    fn into(self) -> Vec2 {
        Vec2::new(self.0.x as f32, self.0.y as f32)
    }
}

impl Into<Vec3> for Position {
    fn into(self) -> Vec3 {
        Vec3::new(self.0.x as f32, self.0.y as f32, 0.0)
    }
}

impl From<Vec2> for Position {
    fn from(v: Vec2) -> Self {
        Position::new(v.x as f64, v.y as f64)
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

impl Div<f64> for Position {
    type Output = Position;

    fn div(self, rhs: f64) -> Self::Output {
        Position::new(self.0.x / &rhs, self.0.y / &rhs)
    }
}

impl Mul<f64> for Position {
    type Output = Position;

    fn mul(self, rhs: f64) -> Self::Output {
        Position::new(self.0.x * &rhs, self.0.y * &rhs)
    }
}

/// -1, 0, 1 on each axis. Since the range is past that we keep the wrapped type private.
#[derive(Debug, Clone, PartialEq)]
pub struct Direction(Vector2<i8>);

impl Direction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn left(&mut self) {
        self.0.x = -1;
    }
    pub fn right(&mut self) {
        self.0.x = 1;
    }
    pub fn up(&mut self) {
        self.0.y = 1;
    }
    pub fn down(&mut self) {
        self.0.y = -1;
    }

    pub fn to_grid_pos(&self) -> GridPosition {
        GridPosition::new(self.0.x as i32, self.0.y as i32)
    }

    pub fn normalized_velocity(&self, speed: &Speed) -> Velocity {
        let mut vel = Velocity::new(self.0.x as f64, self.0.y as f64);
        if vel.0.magnitude() > 0.0 {
            vel.0 = vel.0.normalize();
        }
        vel.0 *= speed.0;
        vel
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self(Vector2::new(0, 0))
    }
}

#[derive(Debug)]
pub struct Speed(pub f64);

impl Speed {
    fn new(speed: f64) -> Self {
        Self(speed)
    }

    pub fn good_guy() -> Self {
        Self::new(0.1)
    }

    pub fn bad_guy() -> Self {
        Self::new(0.04 + thread_rng().gen_range(0.0..0.02))
    }
}

#[derive(Debug, Clone)]
pub struct Velocity(pub Vector2<f64>);

impl Velocity {
    pub fn new(x: f64, y: f64) -> Self {
        Self(Vector2::new(x, y))
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

pub fn check_velocity_collisions(map: Res<PathfindingMap>, mut query: Query<(&Position, &mut Velocity)>) {
    let map: &PathfindingMap = map.deref();
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

pub fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0;
    }
}

pub fn sync_sprite_positions(mut query: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (pos, mut transform) in query.iter_mut() {
        *transform = pos.to_transform();
    }
}
