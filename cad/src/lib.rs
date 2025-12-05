#![allow(
    clippy::uninlined_format_args,
    clippy::too_many_arguments,
    clippy::uninlined_format_args
)]

use std::{error::Error, path::PathBuf};

use curvo::prelude::{NurbsCurve3D, SurfaceTessellation3D};
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};

use crate::{
    sketch::Sketch,
    topology::{Face, Solid},
};

pub mod entity;
pub mod registry;
pub mod sketch;
pub mod topology;

#[derive(Debug, Serialize, Deserialize)]
pub struct Plane {
    pub x: nalgebra::Vector3<f64>,
    pub y: nalgebra::Vector3<f64>,
}

impl Plane {
    pub fn normal(&self) -> nalgebra::Vector3<f64> {
        self.x.cross(&self.y).normalize()
    }

    pub fn origin(&self) -> nalgebra::Vector3<f64> {
        nalgebra::Vector3::zeros()
    }
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

impl SketchInfo {
    #[inline(always)]
    pub fn sketch_space_to_scene_space(&self, v: Vector2<f64>) -> Vector3<f64> {
        self.plane.x * v.x + self.plane.y * v.y
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub path: Option<PathBuf>,
    pub sketches: Vec<SketchInfo>,
    pub solids: Solid,
}

impl Scene {
    pub fn add_sketch(&mut self, plane: Plane) {
        let name = format!("Sketch {}", self.sketches.len() + 1);
        self.sketches.push(SketchInfo {
            id: self.sketches.last().map(|si| si.id + 1).unwrap_or_default(),
            plane,
            sketch: Sketch::new(name.clone()),
            name,
            visible: true,
        });
    }

    pub fn extrude(&mut self, sketch_id: u16, face: Face) -> Solid {
        todo!()
    }

    pub fn loop_to_curve(
        &self,
        sketch_id: u16,
        face: Face,
    ) -> Result<NurbsCurve3D<f64>, Box<dyn Error>> {
        // Only handle capped lines for now. Arcs will be approximated by many points
        // Get the corners in 3D space
        // Construct curve
        let si = self
            .sketches
            .iter()
            .find(|si| si.id == sketch_id)
            .ok_or(format!("Sketch of id {} not found", sketch_id))?;
        let mut points: Vec<entity::Point> = vec![];
        for (i, topo_id) in face.ids.iter().enumerate() {
            match si.sketch.topo_entities[*topo_id] {
                topology::TopoEntity::Edge { edge } => match edge {
                    topology::Edge::CappedLine {
                        start,
                        end,
                        line: _,
                    } => {
                        points.push(si.sketch.geo_entities[start].try_into().unwrap());
                        if i == face.ids.len() - 1 {
                            points.push(si.sketch.geo_entities[end].try_into().unwrap());
                        }
                    }
                    topology::Edge::ArcThreePoint { .. } => {
                        todo!("Arc edges cant be rasterized (yet)")
                    }
                },
                _ => return Err("A face shouldn't contain ids for non-edge entities".into()),
            }
        }

        let curve = NurbsCurve3D::bezier(
            &points
                .into_iter()
                .map(|p| si.sketch_space_to_scene_space(p.pos))
                .collect(),
        );
        Ok(curve)
    }
}
