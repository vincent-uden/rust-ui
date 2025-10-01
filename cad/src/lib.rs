#![allow(
    clippy::uninlined_format_args,
    clippy::too_many_arguments,
    clippy::uninlined_format_args
)]

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::sketch::Sketch;

pub mod entity;
pub mod registry;
pub mod sketch;

#[derive(Debug, Serialize, Deserialize)]
pub struct Plane {
    pub x: nalgebra::Vector3<f64>,
    pub y: nalgebra::Vector3<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SketchInfo {
    /// Stable id that doesnt change even if ordering does in a scene
    pub id: u16,
    pub plane: Plane,
    pub sketch: Sketch,
    pub name: String,
    pub visible: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub path: Option<PathBuf>,
    pub sketches: Vec<SketchInfo>,
}
