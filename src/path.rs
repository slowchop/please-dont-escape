use bevy::prelude::*;
use crate::position::{GridPosition, Position, Velocity, Speed};

#[derive(Debug)]
pub struct Path {
    cells: Vec<GridPosition>,
    current: usize,
}

impl Path {
    pub fn new(cells: &[GridPosition]) -> Self {
        Self {
            cells: cells.into(),
            current: 0,
        }
    }

    fn target(&self) -> &GridPosition {
        &self.cells[self.current]
    }

    fn next(&mut self) -> Option<&GridPosition> {
        let next_idx = self.current + 1;
        if self.cells.len() == next_idx {
            None
        } else {
            self.current = next_idx;
            Some(self.target())
        }
    }
}

pub fn move_along_path(mut query: Query<(&mut Velocity, &mut Path, &Position, &Speed)>) {
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
