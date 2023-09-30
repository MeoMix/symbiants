use super::WorldMap;
use bevy::prelude::*;
use serde::de::{self, Deserializer, MapAccess, Unexpected, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul};
#[derive(Component, Debug, Eq, PartialEq, Hash, Copy, Clone, Reflect, Default)]
#[reflect(Component, Hash, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: String = format!("{},{}", self.x, self.y);
        serializer.serialize_str(&s)
    }
}

struct PositionVisitor;

impl<'de> Visitor<'de> for PositionVisitor {
    type Value = Position;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("either a map with keys 'x' and 'y', or a string in the format 'x,y'")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Position, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut x = None;
        let mut y = None;
        while let Some(key) = map.next_key()? {
            match key {
                "x" => {
                    if x.is_some() {
                        return Err(de::Error::duplicate_field("x"));
                    }
                    x = Some(map.next_value()?);
                }
                "y" => {
                    if y.is_some() {
                        return Err(de::Error::duplicate_field("y"));
                    }
                    y = Some(map.next_value()?);
                }
                _ => return Err(de::Error::unknown_field(key, &["x", "y"])),
            }
        }
        let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
        let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
        Ok(Position { x, y })
    }

    fn visit_str<E>(self, value: &str) -> Result<Position, E>
    where
        E: de::Error,
    {
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() != 2 {
            return Err(E::invalid_value(Unexpected::Str(value), &self));
        }

        let x: isize = parts[0]
            .parse()
            .map_err(|_| E::invalid_value(Unexpected::Str(parts[0]), &self))?;
        let y: isize = parts[1]
            .parse()
            .map_err(|_| E::invalid_value(Unexpected::Str(parts[1]), &self))?;

        Ok(Position { x, y })
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Position, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PositionVisitor)
    }
}

impl Position {
    #[allow(dead_code)]
    pub const ZERO: Self = Self::new(0, 0);
    pub const X: Self = Self::new(1, 0);
    pub const NEG_X: Self = Self::new(-1, 0);

    pub const Y: Self = Self::new(0, 1);
    pub const NEG_Y: Self = Self::new(0, -1);

    pub const ONE: Self = Self::new(1, 1);
    pub const NEG_ONE: Self = Self::new(-1, -1);

    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    // Convert Position to Transform, z-index is naively set to 1 for now
    pub fn as_world_position(&self, world_map: &Res<WorldMap>) -> Vec3 {
        let y_offset = *world_map.height() as f32 / 2.0;
        let x_offset = *world_map.width() as f32 / 2.0;

        Vec3 {
            // NOTE: unit width is 1.0 so add 0.5 to center the position
            x: self.x as f32 - x_offset + 0.5,
            // The view of the model position is just an inversion along the y-axis.
            y: -self.y as f32 + y_offset - 0.5,
            z: 1.0,
        }
    }

    /// Returns the Manhattan Distance between two positions
    pub fn distance(&self, other: &Position) -> isize {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Mul for Position {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}
