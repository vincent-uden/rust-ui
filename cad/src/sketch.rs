use nalgebra::Vector2;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::entity::{
    ArcThreePoint, BiConstraint, CappedLine, Circle, EntityId, FundamentalEntity, GuidedEntity,
    Line, Point,
};
use crate::registry::Registry;
use crate::topology::Loop;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sketch {
    name: String,
    pub fundamental_entities: Registry<EntityId, FundamentalEntity>,
    pub guided_entities: Registry<EntityId, GuidedEntity>,
    pub bi_constraints: Vec<BiConstraint>,
    step_size: f64,
}

impl Sketch {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fundamental_entities: Registry::new(),
            guided_entities: Registry::new(),
            bi_constraints: Vec::new(),
            step_size: 1e-2,
        }
    }

    pub fn error(&self) -> f64 {
        let mut sum = 0.0;
        for BiConstraint { e1, e2, c } in &self.bi_constraints {
            sum += BiConstraint::error(
                &self.fundamental_entities[*e1],
                &self.fundamental_entities[*e2],
                c,
            );
        }
        sum
    }

    pub fn sgd_step(&mut self) {
        let mut rng = rand::rng();
        for BiConstraint { e1, e2, c } in &self.bi_constraints {
            let [fe1, fe2] = self.fundamental_entities.get_disjoint_mut([e1, e2]);
            let fe1 = fe1.unwrap();
            let fe2 = fe2.unwrap();
            if rng.random_bool(0.5) {
                BiConstraint::apply_grad_error(fe1, fe2, c, self.step_size);
            } else {
                BiConstraint::apply_grad_error(fe2, fe1, c, self.step_size);
            }
        }
    }

    fn query_point(&self, query_pos: &Vector2<f64>, radius: f64) -> Option<EntityId> {
        let mut closest_id = None;
        let mut closest_dist = f64::INFINITY;
        for (id, e) in self.fundamental_entities.iter() {
            if let FundamentalEntity::Point { .. } = e {
                let dist = e.distance_to_position(query_pos);
                if dist <= radius && dist < closest_dist {
                    closest_id = Some(*id);
                    closest_dist = dist;
                }
            }
        }
        closest_id
    }

    pub fn query_or_insert_point(&mut self, query_pos: &Vector2<f64>, radius: f64) -> EntityId {
        if let Some(id) = self.query_point(query_pos, radius) {
            id
        } else {
            self.fundamental_entities
                .insert(FundamentalEntity::Point { pos: *query_pos })
        }
    }

    pub fn insert_point(&mut self, pos: Vector2<f64>) -> EntityId {
        let id = self
            .fundamental_entities
            .insert(FundamentalEntity::Point { pos });
        self.guided_entities.insert(GuidedEntity::Point { id })
    }

    /// Inserts `n - 1` lines were `n` is the length of `points`. Each line but the first shares its
    /// starting point with the end point of the preceeding line.
    pub fn insert_capped_lines(&mut self, points: &[Vector2<f64>]) {
        let mut start_id = self
            .fundamental_entities
            .insert(FundamentalEntity::Point { pos: points[0] });
        for w in points.windows(2) {
            let start = w[0];
            let end = w[1];
            let end_id = self
                .fundamental_entities
                .insert(FundamentalEntity::Point { pos: end });
            let line_id = self.fundamental_entities.insert(FundamentalEntity::Line {
                offset: start,
                direction: (end - start),
            });
            self.guided_entities.insert(GuidedEntity::CappedLine {
                start: start_id,
                end: end_id,
                line: line_id,
            });
            start_id = end_id;
        }
    }

    pub fn insert_circle(&mut self, center: Vector2<f64>, radius: f64) {
        let circle_id = self.fundamental_entities.insert(FundamentalEntity::Circle {
            pos: center,
            radius,
        });
        self.guided_entities
            .insert(GuidedEntity::Circle { id: circle_id });
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
        let start_pos = match self.fundamental_entities.get(&line.start) {
            Some(FundamentalEntity::Point { pos }) => *pos,
            _ => return false,
        };
        let end_pos = match self.fundamental_entities.get(&line.end) {
            Some(FundamentalEntity::Point { pos }) => *pos,
            _ => return false,
        };

        let dx = end_pos.x - start_pos.x;
        let dy = end_pos.y - start_pos.y;
        let denom = dx * ray.y - dy * ray.x;

        if denom.abs() < 1e-12 {
            // Parallel or coincident, no intersection
            return false;
        }

        let t = ((start_pos.x - point.x) * ray.y - (start_pos.y - point.y) * ray.x) / denom;
        let s = ((start_pos.x - point.x) * dy - (start_pos.y - point.y) * dx) / denom;

        t >= 0.0 && (0.0..=1.0).contains(&s)
    }

    /// Determines if `point` is inside `l` (assuming `l` is a properly constructed
    /// [Loop]). Algorithm is implemented based on [Containment test for polygons
    /// containing circular arcs](https://ieeexplore.ieee.org/document/1011280).
    pub fn is_inside(&self, l: &Loop, point: Vector2<f64>) -> bool {
        let mut intersections = 0;

        let test_ray = Vector2::x();

        for id in &l.ids {
            match self.guided_entities.get(id) {
                Some(x) => match *x {
                    GuidedEntity::Circle { id: _ } => todo!(),
                    GuidedEntity::CappedLine { start, end, line } => {
                        if self.intersects_capped_line(
                            point,
                            test_ray,
                            &CappedLine { start, end, line },
                        ) {
                            intersections += 1;
                        }
                    }
                    GuidedEntity::ArcThreePoint {
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
                        let circle: Circle = (*self.fundamental_entities.get(&circle).unwrap())
                            .try_into()
                            .unwrap();
                        if (point - circle.pos).norm_squared() > circle.radius.powi(2) {
                            if self.intersects_capped_line(
                                point,
                                test_ray,
                                &CappedLine {
                                    start,
                                    end,
                                    line: EntityId::default(),
                                },
                            ) {
                                intersections += 1;
                            }
                        } else {
                            let start: Point = (*self.fundamental_entities.get(&start).unwrap())
                                .try_into()
                                .unwrap();
                            let middle: Point = (*self.fundamental_entities.get(&middle).unwrap())
                                .try_into()
                                .unwrap();
                            let end: Point = (*self.fundamental_entities.get(&end).unwrap())
                                .try_into()
                                .unwrap();
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
                    GuidedEntity::Point { id: _ } => {
                        error!("A loop can't contain a point");
                    }
                    GuidedEntity::Line { id: _ } => {
                        error!("A loop can't contain a line");
                    }
                },
                None => todo!(),
            }
        }

        intersections % 2 == 1
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
        let e1 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(0.0, 0.0),
            });
        let e2 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
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
        let e1 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(0.0, 0.0),
            });
        let e2 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
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
        let e1 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(0.0, 0.0),
            });
        let e2 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(1.0, 0.1),
            });
        let e3 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
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

        let top_corner = if let FundamentalEntity::Point { pos } = &sketch.fundamental_entities[e3]
        {
            Point { pos: *pos }
        } else {
            panic!("Expected Point");
        };
        let right_corner =
            if let FundamentalEntity::Point { pos } = &sketch.fundamental_entities[e2] {
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
        let e1 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(3.0, 1.0),
            });
        let e2 = sketch.fundamental_entities.insert(FundamentalEntity::Line {
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
        let e1 = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Circle {
                pos: Vector2::new(0.0, -1.0),
                radius: 1.0,
            });
        sketch.dump("Circle Line Tangent Intial");
        let e2 = sketch.fundamental_entities.insert(FundamentalEntity::Line {
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

        match sketch.fundamental_entities[e1] {
            FundamentalEntity::Point { .. } => panic!("e1 should be a circle"),
            FundamentalEntity::Line { .. } => panic!("e1 should be a circle"),
            FundamentalEntity::Circle { pos, radius } => {
                let c = Circle { pos, radius };
                println!("{:?}", c);
                assert!(c.radius > 1e-2, "The radius should be larger than 1e-2")
            }
        }

        match sketch.fundamental_entities[e2] {
            FundamentalEntity::Point { .. } => panic!("e2 should be a line"),
            FundamentalEntity::Line { offset, direction } => {
                let l = Line { offset, direction };
                println!("{:?}", l);
            }
            FundamentalEntity::Circle { .. } => panic!("e2 should be a line"),
        }
    }

    #[test]
    fn rotating_line_test() {
        // The line should rotate and be offset to align with the points called x and y
        let mut sketch = Sketch::new("Rotating Line Sketch".to_string());
        let origin = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(0.0, 0.0),
            });
        let x = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(1.0, 0.0),
            });
        let y = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Point {
                pos: Vector2::new(0.0, -1.0),
            });
        let l1 = sketch.fundamental_entities.insert(FundamentalEntity::Line {
            offset: Vector2::new(0.0, 0.0),
            direction: Vector2::new(1.0, 0.2),
        });
        let l2 = sketch.fundamental_entities.insert(FundamentalEntity::Line {
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
        let c = sketch
            .fundamental_entities
            .insert(FundamentalEntity::Circle {
                pos: Vector2::new(-0.01453125, -0.3746484375),
                radius: 1.1365623545023815,
            });
        let l1 = sketch.fundamental_entities.insert(FundamentalEntity::Line {
            offset: Vector2::new(1.56115234375, 0.7165625),
            direction: Vector2::new(-2.3505859375, 0.7005468749999999),
        });
        let l2 = sketch.fundamental_entities.insert(FundamentalEntity::Line {
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
                let angle = (i as f64 * 360.0 / 5.0);
                Vector2::new(radius * angle.cos(), radius * angle.sin())
            })
            .collect();
        sketch.insert_capped_lines(&corners);
        let l = Loop {
            ids: sketch.guided_entities.keys().cloned().collect(),
        };
        assert_eq!(l.ids.len(), 5);
        assert!(sketch.is_inside(&l, Vector2::new(0.5, 0.5)));
    }
}
