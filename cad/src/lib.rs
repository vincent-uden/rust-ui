use serde::{Deserialize, Serialize};

use crate::sketch::Sketch;

pub mod entity;
pub mod registry;
pub mod sketch;

#[derive(Debug, Serialize, Deserialize)]
pub struct Plane {
    x: nalgebra::Vector3<f64>,
    y: nalgebra::Vector3<f64>,
}

// TODO: Continue defining the scene struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    sketches: Vec<Sketch>,
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
