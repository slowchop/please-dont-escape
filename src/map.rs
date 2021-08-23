use bevy::ecs::prelude::{Added, Or, Query, ResMut};

use crate::game::{Cell, Position};

/// Specific cells that can be walked on. This should be added when NonWalkable was removed.
#[derive(Debug)]
pub struct Walkable;

#[derive(Debug)]
pub struct NonWalkable;

#[derive(Debug)]
pub struct Map {
    walkable_cells: bevy::utils::HashMap<Cell, bool>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            walkable_cells: bevy::utils::HashMap::default(),
        }
    }

    pub(crate) fn is_walkable_pos(&self, pos: &Position) -> bool {
        self.is_walkable_cell(&pos.nearest_cell())
    }

    /// Outside the map is not walkable.
    fn is_walkable_cell(&self, cell: &Cell) -> bool {
        *self.walkable_cells.get(&cell).unwrap_or(&false)
    }

    pub(crate) fn walkable_neighbours(&self, cell: &Cell) -> Vec<Cell> {
        Cell::four_directions()
            .iter()
            .map(|c| cell + c)
            .filter(|c| self.is_walkable_cell(&c))
            .collect()
    }
}

pub fn update_map_with_walkables(
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
