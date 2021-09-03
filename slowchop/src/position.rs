use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::{Add, AddAssign, From, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use fixed::prelude::LossyInto;
use fixed::types::I44F20;
use nalgebra::Vector3;
use std::convert::TryInto;
use std::io::{Error, ErrorKind, Write};
use bevy::prelude::*;

/// The amount of Z snaps per X/Y snap. e.g. Number of vertical steps in a cell block.
const Z_SNAP: u32 = 10;

/// A wrapper around a fixed point value using the `fixed` crate.
#[derive(Debug, Clone, PartialEq, From, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign)]
pub struct Fixed64(I44F20);

impl Fixed64 {
    const LEN: usize = 8;
}

impl From<u64> for Fixed64 {
    fn from(n: u64) -> Self {
        Self(I44F20::from_num(n))
    }
}

impl From<f32> for Fixed64 {
    fn from(n: f32) -> Self {
        Self(I44F20::from_num(n))
    }
}

impl BorshSerialize for Fixed64 {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0.to_bits().to_le_bytes())
    }
}

// This implementation is very similar to existing BorshDeserialize implementations.
impl BorshDeserialize for Fixed64 {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        if buf.len() < Self::LEN {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Not enough bytes when deserializing F64",
            ));
        }

        let res = I44F20::from_le_bytes(buf[..Self::LEN].try_into().unwrap());
        *buf = &buf[Self::LEN..];
        Ok(Self(res))
    }
}

/// Global position for all the things.
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Position {
    x: Fixed64,
    y: Fixed64,
    z: Fixed64,
}

/// A "snapped" cell. Z is snapped differently as it is more granular.
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Cell {
    x: i64,
    y: i64,
    z: i64,
}

impl Cell {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }
}

impl Position {
    pub fn new_u64(x: u64, y: u64, z: u64) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    pub fn new_f32(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    pub fn to_vector3(&self) -> Vector3<f32> {
        Vector3::new(
            self.x.0.lossy_into(),
            self.y.0.lossy_into(),
            self.z.0.lossy_into(),
        )
    }

    pub fn to_vector3_relative_todo(&self, _offset: &Position) -> Vector3<f32> {
        warn!("TODO: relative position");
        Vector3::new(
            self.x.0.lossy_into(),
            self.y.0.lossy_into(),
            self.z.0.lossy_into(),
        )
    }

    pub fn to_cell_floor(&self) -> Cell {
        Cell {
            x: self.x.0.floor().lossy_into(),
            y: self.y.0.floor().lossy_into(),
            z: self.z.0.floor().lossy_into(),
        }
    }

    pub fn to_cell_round(&self) -> Cell {
        Cell {
            x: self.x.0.round().lossy_into(),
            y: self.y.0.round().lossy_into(),
            z: self.z.0.round().lossy_into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};

    #[test]
    fn usage() {
        let mut p1 = Position::new_u64(1u64, 10u64, 0u64);
        let p2 = Position::new_f32(1.5f32, 10f32, 0f32);
        p1.x += 0.5f32.into();

        assert_eq!(p1, p2);
    }

    #[test]
    fn floor() {
        let cell = Position::new_f32(1.9f32, 2.2f32, 3.3f32).to_cell_floor();
        assert_eq!(cell, Cell::new(1i64, 2i64, 3i64));
    }

    #[test]
    fn serialize_position() {
        let a = Position {
            x: Fixed64::from(0),
            y: Fixed64::from(1),
            z: Fixed64::from(-9999.75),
        };
        let v = a.try_to_vec().unwrap();
        let b = Position::try_from_slice(&v).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn serialize_cell() {
        let a = Cell::new(1, 2, 3);
        let v = a.try_to_vec().unwrap();
        let b = Cell::try_from_slice(&v).unwrap();
        assert_eq!(a, b);
    }
}

