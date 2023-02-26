use super::point;
use std::iter::FromIterator;
use uuid::Uuid;

pub enum Behavior {
    Wandering,
    Carrying,
}

pub enum Facing {
    Left,
    Right,
}

// TODO: it's awkward that these aren't numbers
pub enum Angle {
    Zero,
    Ninety,
    OneHundredEighty,
    TwoHundredSeventy,
}

pub struct Ant {
    id: Uuid,
    location: point::Point,
    behavior: Behavior,
    facing: Facing,
    angle: Angle,
    timer: i32,
    name: String,
    active: bool,
}

impl Ant {
    pub fn new(
        x: i32,
        y: i32,
        behavior: Behavior,
        facing: Facing,
        angle: Angle,
        name: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            location: point::Point { x, y },
            behavior,
            facing,
            angle,
            // timer: getTimer(behavior),
            timer: 6,
            name,
            active: true,
        }
    }
}

// impl FromIterator<(i32, i32, Behavior, Facing, Angle, String)> for Ant {
//     fn from_iter<I: IntoIterator<Item = (i32, i32, Behavior, Facing, Angle, String)>>(
//         iter: I,
//     ) -> Self {
//         let ants: Vec<Ant> = iter
//             .into_iter()
//             .map(|(x, y, behavior, facing, angle, name)| {
//                 Ant::new(x, y, behavior, facing, angle, name)
//             })
//             .collect();

//         // Since we are collecting the iterator into a Vec<Ant>, we can just return the first element of the Vec
//         ants.into_iter().next().unwrap()
//     }
// }

// const BehaviorTimingFactors = {
//   wandering: 4,
//   carrying: 5,
// }

// export const getTimer = (behavior: Behavior) => BehaviorTimingFactors[behavior] + Math.floor((Math.random() * 3)) - 1;

// fn getTimer(behavior: Behavior) {
//     6
// }
