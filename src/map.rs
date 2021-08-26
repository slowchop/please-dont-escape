use crate::position::{GridPosition, Position, FlexPosition};
use bevy::prelude::*;
use pathfinding::prelude::astar;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub items: Vec<ItemInfo>,
}

impl Map {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemInfo {
    pub item: Item,
    pub pos: FlexPosition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Background(String),
    Warden,
    Prisoner,
    Wall,
    Door,
    Exit,
    Wire,
}

impl Item {
    pub fn path(&self) -> PathBuf {
        match self {
            Item::Wall => "cells/wall.png".into(),
            Item::Door => "cells/prison-door.png".into(),
            Item::Exit => "cells/exit.png".into(),
            Item::Wire => "cells/wire.png".into(),
            Item::Prisoner => "chars/prisoner.png".into(),
            Item::Warden => "chars/warden.png".into(),
            Item::Background(s) => s.into(),
        }
    }
}

/// Specific cells that can be walked on. This should be added when NonWalkable was removed.
#[derive(Debug)]
pub struct Walkable;

#[derive(Debug)]
pub struct NonWalkable;

#[derive(Debug)]
pub struct PathfindingMap {
    walkable_cells: bevy::utils::HashMap<GridPosition, bool>,
}

impl PathfindingMap {
    pub fn new() -> Self {
        Self {
            walkable_cells: bevy::utils::HashMap::default(),
        }
    }

    pub fn is_walkable_pos(&self, pos: &Position) -> bool {
        self.is_walkable_cell(&pos.nearest_cell())
    }

    /// Outside the map is not walkable.
    fn is_walkable_cell(&self, cell: &GridPosition) -> bool {
        *self.walkable_cells.get(&cell).unwrap_or(&false)
    }

    pub fn walkable_neighbours(&self, cell: &GridPosition) -> Vec<GridPosition> {
        GridPosition::four_directions()
            .iter()
            .map(|c| cell + c)
            .filter(|c| self.is_walkable_cell(&c))
            .collect()
    }

    pub fn find_path(
        &self,
        src: &GridPosition,
        dst: &GridPosition,
    ) -> Option<(Vec<GridPosition>, i32)> {
        astar(
            src,
            |cell: &GridPosition| {
                self.walkable_neighbours(cell)
                    .into_iter()
                    .map(|d| (d, 1i32))
                    .collect::<Vec<_>>()
            },
            |cell| {
                let diff = cell - dst;
                diff.0.x.abs() + diff.0.y.abs()
            },
            |c| c == dst,
        )
    }
}

pub fn update_map_with_walkables(
    mut map: ResMut<PathfindingMap>,
    query: Query<
        (&GridPosition, Option<&NonWalkable>, Option<&Walkable>),
        Or<(Added<NonWalkable>, Added<Walkable>)>,
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
