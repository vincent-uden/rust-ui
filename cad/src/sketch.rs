use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use nalgebra::Vector2;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::entity::{self, BiConstraint, Circle, GeoId, GeometricEntity, Point};
use crate::registry::Registry;
use crate::topology::{
    self, ArcThreePoint, CappedLine, Edge, Loop, ParametrizedIntersection, TopoEntity, TopoId, Wire,
};

const EQ_TOL: f64 = 1e-10;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sketch {
    name: String,
    pub geo_entities: Registry<GeoId, GeometricEntity>,
    pub topo_entities: Registry<TopoId, TopoEntity>,
    pub bi_constraints: Vec<BiConstraint>,
    pub wires: Vec<Wire>,
    step_size: f64,
}

impl Sketch {
    pub fn new(name: String) -> Self {
        Self {
            name,
            geo_entities: Registry::new(),
            topo_entities: Registry::new(),
            bi_constraints: Vec::new(),
            wires: Vec::new(),
            step_size: 1e-2,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut contents = String::new();
        File::open(path)?.read_to_string(&mut contents)?;
        serde_json::from_str(&contents).map_err(|e| Box::from(e))
    }

    pub fn error(&self) -> f64 {
        let mut sum = 0.0;
        for BiConstraint { e1, e2, c } in &self.bi_constraints {
            sum += BiConstraint::error(&self.geo_entities[*e1], &self.geo_entities[*e2], c);
        }
        sum
    }

    pub fn sgd_step(&mut self) {
        let mut rng = rand::rng();
        for BiConstraint { e1, e2, c } in &self.bi_constraints {
            let [fe1, fe2] = self.geo_entities.get_disjoint_mut([e1, e2]);
            let fe1 = fe1.unwrap();
            let fe2 = fe2.unwrap();
            if rng.random_bool(0.5) {
                BiConstraint::apply_grad_error(fe1, fe2, c, self.step_size);
            } else {
                BiConstraint::apply_grad_error(fe2, fe1, c, self.step_size);
            }
        }
    }

    fn query_point(&self, query_pos: &Vector2<f64>, radius: f64) -> Option<GeoId> {
        let mut closest_id = None;
        let mut closest_dist = f64::INFINITY;
        for (id, e) in self.geo_entities.iter() {
            if let GeometricEntity::Point { .. } = e {
                let dist = e.distance_to_position(query_pos);
                if dist <= radius && dist < closest_dist {
                    closest_id = Some(*id);
                    closest_dist = dist;
                }
            }
        }
        closest_id
    }

    pub fn query_or_insert_point(&mut self, query_pos: &Vector2<f64>, radius: f64) -> GeoId {
        if let Some(id) = self.query_point(query_pos, radius) {
            id
        } else {
            self.geo_entities
                .insert(GeometricEntity::Point { pos: *query_pos })
        }
    }

    pub fn insert_point(&mut self, pos: Vector2<f64>) -> TopoId {
        let id = self.geo_entities.insert(GeometricEntity::Point { pos });
        self.topo_entities.insert(TopoEntity::Point { id })
    }

    /// Inserts `n - 1` lines were `n` is the length of `points`. Each line but the first shares its
    /// starting point with the end point of the preceeding line.
    pub fn insert_capped_lines(&mut self, points: &[Vector2<f64>]) -> Vec<TopoId> {
        // TODO: Check for intersections with existing entities
        // If intersecting:
        // - split the line and the other entity in two
        // - apply colinear constraints and so on
        let mut out = vec![];
        let mut start_id = self
            .geo_entities
            .insert(GeometricEntity::Point { pos: points[0] });
        debug!("Points: {:?}", points);
        for w in points.windows(2) {
            let start = w[0];
            let end = w[1];
            let end_id = self
                .geo_entities
                .insert(GeometricEntity::Point { pos: end });
            let line_id = self.geo_entities.insert(GeometricEntity::Line {
                offset: start,
                direction: (end - start),
            });
            let pending_line = CappedLine {
                start: start_id,
                end: end_id,
                line: line_id,
            };

            let mut split_point_ids = vec![start_id];
            for (id, _, split_point) in self.intersecting_capped_lines(pending_line) {
                let (fst, _) = self.split_capped_line(id, split_point);
                split_point_ids.push(fst.end);
            }

            debug!("Splits {:?}", split_point_ids);
            // TODO: Sometimes the constructed has too few splits even though there are the correct
            // amount of split points

            if split_point_ids.len() == 1 {
                // The simple case where we don't intersect any existing lines
                out.push(self.topo_entities.insert(pending_line.into()));
            } else {
                // The original line is no longer needed since we'll create one for each split segment
                self.geo_entities.remove(&line_id);
                split_point_ids.push(end_id);
                for w in split_point_ids.windows(2) {
                    let start: entity::Point = self.geo_entities[w[0]].try_into().unwrap();
                    let end: entity::Point = self.geo_entities[w[1]].try_into().unwrap();

                    let paritial_line_id = self.geo_entities.insert(GeometricEntity::Line {
                        offset: start.pos,
                        direction: (end.pos - start.pos),
                    });

                    let partial_line = CappedLine {
                        start: w[0],
                        end: w[1],
                        line: paritial_line_id,
                    };
                    self.topo_entities.insert(partial_line.into());
                }
            }
            start_id = end_id;
        }
        out
    }

    pub fn insert_circle(&mut self, center: Vector2<f64>, radius: f64) {
        // TODO: Check for intersections with existing entities
        // If intersecting:
        // - split the line and the other entity in two
        // - apply colinear constraints and so on
        let circle_id = self.geo_entities.insert(GeometricEntity::Circle {
            pos: center,
            radius,
        });
        self.topo_entities
            .insert(TopoEntity::Circle { id: circle_id });
    }

    #[allow(unused)]
    fn dump(&self, name: &str) {
        let mut file = std::fs::File::create(format!(
            "./test_sketches/{}.json",
            name.replace(" ", "_").to_lowercase()
        ))
        .unwrap();
        serde_json::to_writer_pretty(&mut file, &self).expect("Failed to write sketch to file");
    }

    fn intersects_capped_line(
        &self,
        point: Vector2<f64>,
        ray: Vector2<f64>,
        line: &CappedLine,
    ) -> bool {
        let (line_start, line_ray) = line.parametrize(&self.geo_entities);

        let denom = ray.x * line_ray.y - ray.y * line_ray.x;
        if denom.abs() < 1e-12 {
            // Can't intersect with a horizontal line
            return false;
        }

        let t =
            ((line_start.x - point.x) * line_ray.y - (line_start.y - point.y) * line_ray.x) / denom;
        let s = ((line_start.x - point.x) * ray.y - (line_start.y - point.y) * ray.x) / denom;

        t > 0.0 && s > 0.0 && s < 1.0
    }

    /// Determines if `point` is inside `l` (assuming `l` is a properly constructed
    /// [Loop]). Algorithm is implemented based on [Containment test for polygons
    /// containing circular arcs](https://ieeexplore.ieee.org/document/1011280).
    pub fn is_inside(&self, l: &Loop, point: Vector2<f64>) -> bool {
        let mut intersections = 0;

        let test_ray = Vector2::x();

        for id in &l.ids {
            match self.topo_entities.get(id) {
                Some(x) => match *x {
                    TopoEntity::Circle { id: _ } => todo!(),
                    TopoEntity::Edge { edge } => match edge {
                        Edge::CappedLine { start, end, line } => {
                            if self.intersects_capped_line(
                                point,
                                test_ray,
                                &CappedLine { start, end, line },
                            ) {
                                intersections += 1;
                            }
                        }
                        Edge::ArcThreePoint {
                            start,
                            middle,
                            end,
                            circle,
                        } => {
                            let arc = ArcThreePoint {
                                start,
                                middle,
                                end,
                                circle,
                            };
                            let circle: Circle = (*self.geo_entities.get(&circle).unwrap())
                                .try_into()
                                .unwrap();
                            if (point - circle.pos).norm_squared() > circle.radius.powi(2) {
                                if self.intersects_capped_line(
                                    point,
                                    test_ray,
                                    &CappedLine {
                                        start,
                                        end,
                                        line: GeoId::default(),
                                    },
                                ) {
                                    intersections += 1;
                                }
                            } else {
                                let start: Point = (*self.geo_entities.get(&start).unwrap())
                                    .try_into()
                                    .unwrap();
                                let middle: Point = (*self.geo_entities.get(&middle).unwrap())
                                    .try_into()
                                    .unwrap();
                                let end: Point =
                                    (*self.geo_entities.get(&end).unwrap()).try_into().unwrap();
                                // Cross product < 0 means middle point is to the right of chord vector,
                                // indicating the arc bulges to the right
                                let chord_dir = end.pos - start.pos;
                                let mid_vec = middle.pos - start.pos;
                                let cross = chord_dir.x * mid_vec.y - chord_dir.y * mid_vec.x;
                                let ymax = start.pos.y.max(end.pos.y);
                                let ymin = start.pos.y.min(end.pos.y);
                                if point.y > ymin && point.y < ymax {
                                    if cross < 0.0 {
                                        intersections += 1
                                    }
                                } else {
                                    if cross >= 0.0 {
                                        intersections += 1
                                    }
                                }
                            }
                        }
                    },
                    TopoEntity::Point { id: _ } => {
                        error!("A loop can't contain a point");
                    }
                    TopoEntity::Line { id: _ } => {
                        error!("A loop can't contain a line");
                    }
                },
                None => todo!(),
            }
        }

        intersections % 2 == 1
    }

    fn capped_line_intersection(
        &self,
        l1: CappedLine,
        l2: CappedLine,
    ) -> Option<ParametrizedIntersection> {
        let (p1, v1) = l1.parametrize(&self.geo_entities);
        let (p2, v2) = l2.parametrize(&self.geo_entities);

        let denom = v1.x * v2.y - v1.y * v2.x;
        if denom.abs() < 1e-12 {
            return None;
        }

        let t = ((p2.x - p1.x) * v2.y - (p2.y - p1.y) * v2.x) / denom;
        let s = ((p2.x - p1.x) * v1.y - (p2.y - p1.y) * v1.x) / denom;

        if t >= EQ_TOL && t <= (1.0 - EQ_TOL) && s >= EQ_TOL && s <= (1.0 - EQ_TOL) {
            Some(ParametrizedIntersection {
                point: Point { pos: p1 + v1 * t },
                t,
                s,
            })
        } else {
            None
        }
    }

    pub fn does_capped_line_intersect_capped_line(&self, l1: CappedLine, l2: CappedLine) -> bool {
        self.capped_line_intersection(l1, l2).is_some()
    }

    fn loops(&self) -> impl Iterator<Item = Loop> {
        self.wires
            .iter()
            .filter_map(|x| x.clone().try_into(&self.topo_entities).ok())
    }

    /// Returns tuples of lines that are intersected and the point of intersection
    fn intersecting_capped_lines(&self, line: CappedLine) -> Vec<(TopoId, CappedLine, Point)> {
        let mut intersections: Vec<_> = self
            .topo_entities
            .iter()
            .filter_map(|(k, v)| match v {
                TopoEntity::Edge {
                    edge: edge @ Edge::CappedLine { .. },
                } => {
                    let as_line = (*edge)
                        .try_into()
                        .expect("Existing line must be a capped line");
                    self.capped_line_intersection(line, as_line)
                        .map(|intersection| (*k, as_line, intersection))
                }
                _ => None,
            })
            .collect();
        intersections.sort_by(|(_, _, i_a), (_, _, i_b)| i_a.t.total_cmp(&i_b.t));
        intersections
            .into_iter()
            .map(|(k, l, i)| (k, l, i.point))
            .collect()
    }

    /// To avoid invalidating [EntityId]s to `existing`, existing will be truncated to be the starting
    /// segment whereas the second half will be a new line.
    fn split_capped_line(
        &mut self,
        existing_id: TopoId,
        split_point: Point,
    ) -> (CappedLine, CappedLine) {
        // TODO: coincident constraint on point and co-linear on snd_line
        let existing = self.topo_entities.get_mut(&existing_id).unwrap();
        let mut existing_line: CappedLine = existing
            .clone()
            .try_into()
            .expect("existing must be a CappedLine");
        let start_point: Point = (*self.geo_entities.get(&existing_line.start).unwrap())
            .try_into()
            .unwrap();
        let end_point: Point = (*self.geo_entities.get(&existing_line.end).unwrap())
            .try_into()
            .unwrap();

        let start = existing_line.start;
        let middle = self.geo_entities.insert(GeometricEntity::Point {
            pos: split_point.pos,
        });
        let second_line = self.geo_entities.insert(GeometricEntity::Line {
            offset: start_point.pos,
            direction: (end_point.pos - start_point.pos),
        });
        let end = existing_line.end;
        *existing = (CappedLine {
            start,
            end: middle,
            line: existing_line.line,
        })
        .into();

        let snd = self.topo_entities.insert(
            (CappedLine {
                start: middle,
                end,
                line: second_line,
            })
            .into(),
        );

        existing_line.end = middle;
        let new_line: CappedLine = (*self.topo_entities.get(&snd).unwrap()).try_into().unwrap();
        (existing_line, new_line)
    }
}

/// ./test_sketches need to exist before running tests. Eventually I'll figure out a test runner
/// that handles this. Cargo sucks
#[cfg(test)]
mod tests {
    use nalgebra::Vector2;
    use tracing::debug;

    use crate::entity::{Circle, ConstraintType, Line, Point};

    use super::*;

    #[cfg(test)]
    impl Drop for Sketch {
        fn drop(&mut self) {
            self.dump(&self.name.replace(" ", "_"));
        }
    }

    #[test]
    fn basic_error_setup() {
        let mut sketch = Sketch::new("Basic Error Setup".to_string());
        let e1 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(0.0, 0.0),
        });
        let e2 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(1.0, 1.0),
        });
        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2,
            c: ConstraintType::Horizontal,
        });

        assert!(sketch.error() > 0.0, "The error should be larger than 0")
    }

    #[test]
    fn basic_grad_error_setup() {
        let mut sketch = Sketch::new("Basic Grad Error Setup".to_string());
        let e1 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(0.0, 0.0),
        });
        let e2 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(1.0, 1.0),
        });
        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2,
            c: ConstraintType::Horizontal,
        });

        let initial_error = sketch.error();
        for _ in 0..20000 {
            sketch.sgd_step();
        }
        let final_error = sketch.error();
        assert!(
            final_error < initial_error,
            "The final error should be smaller than the intial error"
        );
        assert!(final_error < 1e-2, "final_error {}", final_error);
    }

    #[test]
    fn pythagorean_triplet() {
        let mut sketch = Sketch::new("Pythagorean Triplet".to_string());
        let e1 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(0.0, 0.0),
        });
        let e2 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(1.0, 0.1),
        });
        let e3 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(0.1, 1.0),
        });

        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2,
            c: ConstraintType::Horizontal,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2: e3,
            c: ConstraintType::Vertical,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2,
            c: ConstraintType::Distance { x: 3.0 },
        });
        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2: e3,
            c: ConstraintType::Distance { x: 4.0 },
        });

        for _ in 0..20000 {
            sketch.sgd_step();
        }

        let top_corner = if let GeometricEntity::Point { pos } = &sketch.geo_entities[e3] {
            Point { pos: *pos }
        } else {
            panic!("Expected Point");
        };
        let right_corner = if let GeometricEntity::Point { pos } = &sketch.geo_entities[e2] {
            Point { pos: *pos }
        } else {
            panic!("Expected Point");
        };
        let diff = (top_corner.pos - right_corner.pos).norm();
        assert!((diff - 5.0) < 1e-6);

        assert!(
            sketch.error() < 1e-6,
            "The error should be smaller than 1e-6"
        );
    }

    #[test]
    fn point_line_coincident() {
        let mut sketch = Sketch::new("Point Line Coincident".to_string());
        let e1 = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(3.0, 1.0),
        });
        let e2 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: Vector2::new(1.0, 1.2),
            direction: Vector2::new(-1.0, -1.0),
        });
        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2,
            c: ConstraintType::Coincident,
        });

        sketch.dump("Point Line Coincident Intial");

        assert!(sketch.error() > 0.0, "The error should be larger than 0");
        for _ in 0..20000 {
            sketch.sgd_step();
        }

        assert!(
            sketch.error() < 1e-6,
            "The error should be smaller than 1e-6"
        );
    }

    #[test]
    fn circle_line_tangent() {
        let mut sketch = Sketch::new("Circle Line Tangent".to_string());
        let e1 = sketch.geo_entities.insert(GeometricEntity::Circle {
            pos: Vector2::new(0.0, -1.0),
            radius: 1.0,
        });
        sketch.dump("Circle Line Tangent Intial");
        let e2 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: Vector2::new(1.0, 1.0),
            direction: Vector2::new(1.0, -1.0),
        });
        sketch.bi_constraints.push(BiConstraint {
            e1,
            e2,
            c: ConstraintType::Tangent,
        });
        sketch.dump("Circle Line Tangent Intial");

        assert!(sketch.error() > 0.0, "The error should be larger than 0");
        for _ in 0..20000 {
            sketch.sgd_step();
        }

        assert!(
            sketch.error() < 1e-6,
            "The error should be smaller than 1e-6"
        );

        match sketch.geo_entities[e1] {
            GeometricEntity::Point { .. } => panic!("e1 should be a circle"),
            GeometricEntity::Line { .. } => panic!("e1 should be a circle"),
            GeometricEntity::Circle { pos, radius } => {
                let c = Circle { pos, radius };
                println!("{:?}", c);
                assert!(c.radius > 1e-2, "The radius should be larger than 1e-2")
            }
        }

        match sketch.geo_entities[e2] {
            GeometricEntity::Point { .. } => panic!("e2 should be a line"),
            GeometricEntity::Line { offset, direction } => {
                let l = Line { offset, direction };
                println!("{:?}", l);
            }
            GeometricEntity::Circle { .. } => panic!("e2 should be a line"),
        }
    }

    #[test]
    fn rotating_line_test() {
        // The line should rotate and be offset to align with the points called x and y
        let mut sketch = Sketch::new("Rotating Line Sketch".to_string());
        let origin = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(0.0, 0.0),
        });
        let x = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(1.0, 0.0),
        });
        let y = sketch.geo_entities.insert(GeometricEntity::Point {
            pos: Vector2::new(0.0, -1.0),
        });
        let l1 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: Vector2::new(0.0, 0.0),
            direction: Vector2::new(1.0, 0.2),
        });
        let l2 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: Vector2::new(1.0, 1.0),
            direction: Vector2::new(0.2, 1.0),
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: origin,
            e2: x,
            c: ConstraintType::Horizontal,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: origin,
            e2: x,
            c: ConstraintType::Distance { x: 1.0 },
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: origin,
            e2: y,
            c: ConstraintType::Vertical,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: origin,
            e2: y,
            c: ConstraintType::Distance { x: 1.0 },
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: origin,
            e2: l1,
            c: ConstraintType::Coincident,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: x,
            e2: l1,
            c: ConstraintType::Coincident,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: x,
            e2: l2,
            c: ConstraintType::Coincident,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: y,
            e2: l2,
            c: ConstraintType::Coincident,
        });
        sketch.dump("Rotating Line Intial");

        assert!(sketch.error() > 0.0, "The error should be larger than 0");
        for _ in 0..20000 {
            sketch.sgd_step();
        }

        assert!(
            sketch.error() < 1e-6,
            "The error should be smaller than 1e-6"
        );
        sketch.dump("Rotating Line After");
    }

    #[test]
    fn circle_tangent_with_two_lines() {
        let mut sketch = Sketch::new("Circle Tangent With Two Lines Sketch".to_string());
        let c = sketch.geo_entities.insert(GeometricEntity::Circle {
            pos: Vector2::new(-0.01453125, -0.3746484375),
            radius: 1.1365623545023815,
        });
        let l1 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: Vector2::new(1.56115234375, 0.7165625),
            direction: Vector2::new(-2.3505859375, 0.7005468749999999),
        });
        let l2 = sketch.geo_entities.insert(GeometricEntity::Line {
            offset: Vector2::new(-1.03939453125, -2.0318359375),
            direction: Vector2::new(2.04685546875, 0.4622460937499999),
        });

        sketch.bi_constraints.push(BiConstraint {
            e1: c,
            e2: l1,
            c: ConstraintType::Tangent,
        });
        sketch.bi_constraints.push(BiConstraint {
            e1: c,
            e2: l2,
            c: ConstraintType::Tangent,
        });
        sketch.dump("Circle Tangent With Two Lines Sketch Initial");

        assert!(sketch.error() > 0.0, "The error should be larger than 0");
        for _ in 0..20000 {
            sketch.sgd_step();
        }

        assert!(
            sketch.error() < 1e-6,
            "The error should be smaller than 1e-6"
        );
    }

    #[test]
    fn point_is_inside_polygon_of_lines() {
        let mut sketch = Sketch::new("Pentagon".to_string());
        let radius = 3.0;
        let corners: Vec<_> = (0..6)
            .map(|i| {
                let angle = (i as f64 * 360.0 / 5.0).to_radians();
                Vector2::new(radius * angle.cos(), radius * angle.sin())
            })
            .collect();
        sketch.insert_capped_lines(&corners);
        let l = Loop {
            ids: sketch.topo_entities.keys().cloned().collect(),
        };
        assert_eq!(l.ids.len(), 5);
        assert!(sketch.is_inside(&l, Vector2::new(0.5, 0.5)));
    }

    #[test]
    fn capped_lines_should_intersect() {
        // Capped lines have to be manually constructed since the method splits any existing lines
        // for loop construction purposes.
        let mut sketch = Sketch::new("Crossed Capped Lines".to_string());
        let l1_points = [Vector2::new(1.0, 1.0), Vector2::new(-1.0, -1.0)];
        let l2_points = [Vector2::new(-1.0, 1.0), Vector2::new(1.0, -1.0)];

        let l1_p0 = sketch.insert_point(l1_points[0]);
        let l1_p1 = sketch.insert_point(l1_points[1]);
        let l2_p0 = sketch.insert_point(l2_points[0]);
        let l2_p1 = sketch.insert_point(l2_points[1]);

        let l1_start: topology::Point = sketch.topo_entities[l1_p0].try_into().unwrap();
        let l1_end: topology::Point = sketch.topo_entities[l1_p1].try_into().unwrap();
        let l2_start: topology::Point = sketch.topo_entities[l2_p0].try_into().unwrap();
        let l2_end: topology::Point = sketch.topo_entities[l2_p1].try_into().unwrap();

        let l1_id = sketch.topo_entities.insert(
            Edge::CappedLine {
                start: l1_start.id,
                end: l1_end.id,
                line: GeoId::default(),
            }
            .into(),
        );
        let l2_id = sketch.topo_entities.insert(
            Edge::CappedLine {
                start: l2_start.id,
                end: l2_end.id,
                line: GeoId::default(),
            }
            .into(),
        );

        let l1 = (*sketch.topo_entities.get(&l1_id).unwrap())
            .try_into()
            .unwrap();
        let l2 = (*sketch.topo_entities.get(&l2_id).unwrap())
            .try_into()
            .unwrap();

        assert!(sketch.does_capped_line_intersect_capped_line(l1, l2))
    }
}
