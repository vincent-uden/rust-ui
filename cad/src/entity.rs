use enum_variant_type::EnumVariantType;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::registry::RegId;

#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize, Serialize,
)]
pub struct GeoId(pub u16);
impl RegId for GeoId {
    /// We start at 1 to allow for the usage of 0 as a "null" id
    fn new() -> Self {
        Self(1)
    }

    fn increment(self) -> Self {
        let GeoId(id) = self;
        Self(id + 1)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, EnumVariantType)]
#[evt(derive(Debug, Deserialize, Serialize, Clone, Copy))]
pub enum GeometricEntity {
    Point {
        pos: Vector2<f64>,
    },
    Line {
        offset: Vector2<f64>,
        direction: Vector2<f64>,
    },
    Circle {
        pos: Vector2<f64>,
        radius: f64,
    },
}

impl Into<nalgebra::Point2<f64>> for Point {
    fn into(self) -> nalgebra::Point2<f64> {
        nalgebra::Point2::new(self.pos.x, self.pos.y)
    }
}

pub fn vector_angle(a: Vector2<f64>) -> f64 {
    let angle = f64::atan2(a.y, a.x);
    if angle < 0.0 {
        angle + 2.0 * std::f64::consts::PI
    } else {
        angle
    }
}

impl GeometricEntity {
    pub fn distance_to_position(&self, target: &Vector2<f64>) -> f64 {
        match self {
            GeometricEntity::Point { pos } => (pos - target).norm(),
            GeometricEntity::Line { offset, direction } => {
                let ortho_a = target - project(target, direction);
                let ortho_r = offset - project(offset, direction);
                (ortho_r - ortho_a).norm()
            }
            GeometricEntity::Circle { pos, radius } => ((target - pos).norm() - radius).abs(),
        }
    }

    pub fn circle_from_three_coords(
        p1: &Vector2<f64>,
        p2: &Vector2<f64>,
        p3: &Vector2<f64>,
    ) -> Option<Self> {
        let temp = p2.norm_squared();
        let bc = (p1.x.powi(2) + p1.y.powi(2) - temp) / 2.0;
        let cd = (temp - p3.x.powi(2) - p3.y.powi(2)) / 2.0;
        let det = (p1.x - p2.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p2.y);
        if det.abs() < 1e-12 {
            return None;
        }

        let center = Vector2::new(
            (bc * (p2.y - p3.y) - cd * (p1.y - p2.y)) / det,
            ((p1.x - p2.x) * cd - (p2.x - p3.x) * bc) / det,
        );

        let radius = ((center.x - p1.x).powi(2) + (center.y - p1.y).powi(2)).sqrt();

        Some(GeometricEntity::Circle {
            pos: center,
            radius,
        })
    }
}

pub fn project(a: &Vector2<f64>, b: &Vector2<f64>) -> Vector2<f64> {
    a.dot(b) / b.dot(b) * b
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ConstraintType {
    Angle { x: f64 },
    Coincident,
    Colinear,            // Should be paired with a parallel constraint for line-line
    Distance { x: f64 }, // Should be paired with a parallel constraint for line-line
    Horizontal,
    Parallel,
    Perpendicular,
    Tangent,
    Vertical,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct BiConstraint {
    pub e1: GeoId,
    pub e2: GeoId,
    pub c: ConstraintType,
}

impl BiConstraint {
    pub fn new(e1: GeoId, e2: GeoId, c: ConstraintType) -> Self {
        Self { e1, e2, c }
    }

    pub fn possible(e1: &GeometricEntity, e2: &GeometricEntity, c: &ConstraintType) -> bool {
        match (e1, e2) {
            (GeometricEntity::Point { .. }, GeometricEntity::Point { .. }) => matches!(
                c,
                ConstraintType::Coincident
                    | ConstraintType::Distance { .. }
                    | ConstraintType::Vertical
                    | ConstraintType::Horizontal
            ),
            (GeometricEntity::Point { .. }, GeometricEntity::Line { .. }) => matches!(
                c,
                ConstraintType::Coincident | ConstraintType::Distance { .. }
            ),
            (GeometricEntity::Point { .. }, GeometricEntity::Circle { .. }) => matches!(
                c,
                ConstraintType::Coincident
                    | ConstraintType::Distance { .. }
                    | ConstraintType::Vertical
                    | ConstraintType::Horizontal
            ),
            (GeometricEntity::Line { .. }, GeometricEntity::Line { .. }) => matches!(
                c,
                ConstraintType::Parallel
                    | ConstraintType::Perpendicular
                    | ConstraintType::Colinear
                    | ConstraintType::Distance { .. }
                    | ConstraintType::Angle { .. }
            ),
            (GeometricEntity::Circle { .. }, GeometricEntity::Line { .. }) => matches!(
                c,
                ConstraintType::Coincident
                    | ConstraintType::Tangent
                    | ConstraintType::Distance { .. }
            ),
            (GeometricEntity::Circle { .. }, GeometricEntity::Circle { .. }) => matches!(
                c,
                ConstraintType::Coincident
                    | ConstraintType::Distance { .. }
                    | ConstraintType::Tangent
                    | ConstraintType::Vertical
                    | ConstraintType::Horizontal
            ),
            (_, _) => BiConstraint::possible(e2, e1, c),
        }
    }

    pub fn error(e1: &GeometricEntity, e2: &GeometricEntity, c: &ConstraintType) -> f64 {
        if Self::possible(e1, e2, c) {
            match (e1, e2) {
                (GeometricEntity::Point { .. }, GeometricEntity::Point { .. }) => {
                    Self::error_pp(e1, e2, *c)
                }
                (GeometricEntity::Point { .. }, GeometricEntity::Line { .. }) => {
                    Self::error_pl(e1, e2, *c)
                }
                (GeometricEntity::Point { .. }, GeometricEntity::Circle { .. }) => {
                    Self::error_pc(e1, e2, *c)
                }
                (GeometricEntity::Line { .. }, GeometricEntity::Line { .. }) => {
                    Self::error_ll(e1, e2, *c)
                }
                (GeometricEntity::Line { .. }, GeometricEntity::Circle { .. }) => {
                    Self::error_lc(e1, e2, *c)
                }
                (GeometricEntity::Circle { .. }, GeometricEntity::Circle { .. }) => {
                    Self::error_cc(e1, e2, *c)
                }
                _ => Self::error(e2, e1, c),
            }
        } else {
            0.0
        }
    }

    fn error_pp<T>(p1: &T, p2: &T, c: ConstraintType) -> f64
    where
        T: TryInto<Point, Error: Debug> + Clone + Copy,
    {
        let p1: Point = (*p1).try_into().unwrap();
        let p2: Point = (*p2).try_into().unwrap();
        match c {
            ConstraintType::Coincident => (p1.pos - p2.pos).norm_squared(),
            ConstraintType::Horizontal => (p1.pos.y - p2.pos.y).powi(2),
            ConstraintType::Vertical => (p1.pos.x - p2.pos.x).powi(2),
            ConstraintType::Distance { x } => ((p1.pos - p2.pos).norm() - x).powi(2),
            _ => 0.0,
        }
    }

    fn error_pl<P, L>(p: &P, l: &L, c: ConstraintType) -> f64
    where
        P: TryInto<Point, Error: Debug> + Clone + Copy,
        L: TryInto<Line, Error: Debug> + Clone + Copy,
    {
        let p: Point = (*p).try_into().unwrap();
        let l: Line = (*l).try_into().unwrap();
        let ortho_a = p.pos - project(&p.pos, &l.direction);
        let mut ortho_r = (p.pos - l.offset) - project(&(p.pos - l.offset), &l.direction);
        if ortho_r.dot(&ortho_a) < 0.0 {
            ortho_r = -ortho_r;
        }
        match c {
            ConstraintType::Coincident => (ortho_r + ortho_a).norm_squared(),
            ConstraintType::Distance { x } => ((ortho_r - ortho_a).norm() - x).powi(2),
            _ => 0.0,
        }
    }

    fn error_pc<P, C>(p: &P, ci: &C, c: ConstraintType) -> f64
    where
        P: TryInto<Point, Error: Debug> + Clone + Copy,
        C: TryInto<Circle, Error: Debug> + Clone + Copy,
    {
        let p: Point = (*p).try_into().unwrap();
        let ci: Circle = (*ci).try_into().unwrap();
        match c {
            ConstraintType::Coincident => ((p.pos - ci.pos).norm() - ci.radius).powi(2),
            ConstraintType::Horizontal => (p.pos.y - ci.pos.y).powi(2),
            ConstraintType::Vertical => (p.pos.x - ci.pos.x).powi(2),
            ConstraintType::Distance { x } => ((p.pos - ci.pos).norm() - x).powi(2),
            _ => 0.0,
        }
    }

    fn error_ll<T>(l1: &T, l2: &T, c: ConstraintType) -> f64
    where
        T: TryInto<Line, Error: Debug> + Clone + Copy,
    {
        let l1: Line = (*l1).try_into().unwrap();
        let l2: Line = (*l2).try_into().unwrap();
        match c {
            ConstraintType::Parallel => (l1.direction.angle(&l2.direction)).powi(2),
            ConstraintType::Perpendicular => {
                (l1.direction.angle(&l2.direction) - std::f64::consts::PI / 2.0).powi(2)
            }
            ConstraintType::Colinear => {
                let ortho_1 = l1.offset - project(&l1.offset, &l1.direction);
                let ortho_2 = l2.offset - project(&l2.offset, &l2.direction);
                (ortho_1 - ortho_2).norm()
            }
            ConstraintType::Distance { x } => {
                let ortho_1 = l1.offset - project(&l1.offset, &l1.direction);
                let ortho_2 = l2.offset - project(&l2.offset, &l2.direction);
                ((ortho_1 - ortho_2).norm() - x).powi(2)
            }
            ConstraintType::Angle { x } => (l1.direction.angle(&l2.direction) - x).powi(2),
            _ => 0.0,
        }
    }

    fn error_lc<L, C>(l: &L, ci: &C, c: ConstraintType) -> f64
    where
        L: TryInto<Line, Error: Debug> + Clone + Copy,
        C: TryInto<Circle, Error: Debug> + Clone + Copy,
    {
        let l: Line = (*l).try_into().unwrap();
        let ci: Circle = (*ci).try_into().unwrap();
        let diff = ci.pos - l.offset;
        let ortho = diff - project(&diff, &l.direction);
        match c {
            // Doesnt seem to be working
            ConstraintType::Tangent => ((ortho).norm() - ci.radius).powi(2),
            ConstraintType::Distance { x } => ((ortho).norm() - x).powi(2),
            _ => 0.0,
        }
    }

    fn error_cc<T>(c1: &T, c2: &T, c: ConstraintType) -> f64
    where
        T: TryInto<Circle, Error: Debug> + Clone + Copy,
    {
        let c1: Circle = (*c1).try_into().unwrap();
        let c2: Circle = (*c2).try_into().unwrap();
        match c {
            ConstraintType::Coincident => (c1.pos - c2.pos).norm_squared(),
            ConstraintType::Horizontal => (c1.pos.y - c2.pos.y).powi(2),
            ConstraintType::Vertical => (c1.pos.x - c2.pos.x).powi(2),
            ConstraintType::Tangent => ((c1.pos - c2.pos).norm() - (c1.radius + c2.radius)).powi(2),
            ConstraintType::Distance { x } => ((c1.pos - c2.pos).norm() - x).powi(2),
            _ => 0.0,
        }
    }

    pub fn apply_grad_error(
        e1: &mut GeometricEntity,
        e2: &GeometricEntity,
        c: &ConstraintType,
        step_size: f64,
    ) {
        match e1 {
            GeometricEntity::Point { pos } => Self::apply_grad_error_p(pos, e2, *c, step_size),
            GeometricEntity::Line { offset, direction } => {
                Self::apply_grad_error_l(offset, direction, e2, *c, step_size)
            }
            GeometricEntity::Circle { pos, radius } => {
                Self::apply_grad_error_c(pos, radius, e2, *c, step_size)
            }
        }
    }

    fn apply_grad_error_p(
        p1_pos: &mut Vector2<f64>,
        e: &GeometricEntity,
        c: ConstraintType,
        step_size: f64,
    ) {
        let h = 1e-6;
        let x_errors = [
            Self::error(
                &GeometricEntity::Point {
                    pos: *p1_pos + Vector2::new(-h / 2.0, 0.0),
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Point {
                    pos: *p1_pos + Vector2::new(h / 2.0, 0.0),
                },
                e,
                &c,
            ),
        ];
        let y_errors = [
            Self::error(
                &GeometricEntity::Point {
                    pos: *p1_pos + Vector2::new(0.0, -h / 2.0),
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Point {
                    pos: *p1_pos + Vector2::new(0.0, h / 2.0),
                },
                e,
                &c,
            ),
        ];

        let x_derivative = (x_errors[1] - x_errors[0]) / h;
        let y_derivative = (y_errors[1] - y_errors[0]) / h;
        let step = Vector2::new(x_derivative, y_derivative);
        *p1_pos -= step * step_size;
    }

    fn apply_grad_error_l(
        l_offset: &mut Vector2<f64>,
        l_direction: &mut Vector2<f64>,
        e: &GeometricEntity,
        c: ConstraintType,
        step_size: f64,
    ) {
        let h = 1e-4;
        let o_x_errors = [
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset + Vector2::new(-h / 2.0, 0.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset + Vector2::new(h / 2.0, 0.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
        ];
        let o_y_errors = [
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset + Vector2::new(0.0, -h / 2.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset + Vector2::new(0.0, h / 2.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
        ];
        let d_x_errors = [
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset,
                    direction: *l_direction + Vector2::new(-h / 2.0, 0.0),
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset,
                    direction: *l_direction + Vector2::new(h / 2.0, 0.0),
                },
                e,
                &c,
            ),
        ];
        let d_y_errors = [
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset,
                    direction: *l_direction + Vector2::new(0.0, -h / 2.0),
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Line {
                    offset: *l_offset,
                    direction: *l_direction + Vector2::new(0.0, h / 2.0),
                },
                e,
                &c,
            ),
        ];

        let o_x_derivative = (o_x_errors[1] - o_x_errors[0]) / h;
        let o_y_derivative = (o_y_errors[1] - o_y_errors[0]) / h;
        let offset_step = Vector2::new(o_x_derivative, o_y_derivative);
        let d_x_derivative = (d_x_errors[1] - d_x_errors[0]) / h;
        let d_y_derivative = (d_y_errors[1] - d_y_errors[0]) / h;
        let direction_step = Vector2::new(d_x_derivative, d_y_derivative);
        *l_offset -= offset_step * step_size;
        *l_direction -= direction_step * step_size;
    }

    fn apply_grad_error_c(
        c1_pos: &mut Vector2<f64>,
        c1_radius: &mut f64,
        e: &GeometricEntity,
        c: ConstraintType,
        step_size: f64,
    ) {
        let h = 1e-6;
        let x_errors = [
            Self::error(
                &GeometricEntity::Circle {
                    pos: *c1_pos + Vector2::new(-h / 2.0, 0.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Circle {
                    pos: *c1_pos + Vector2::new(h / 2.0, 0.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
        ];
        let y_errors = [
            Self::error(
                &GeometricEntity::Circle {
                    pos: *c1_pos + Vector2::new(0.0, -h / 2.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Circle {
                    pos: *c1_pos + Vector2::new(0.0, h / 2.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
        ];
        let r_errors = [
            Self::error(
                &GeometricEntity::Circle {
                    pos: *c1_pos,
                    radius: *c1_radius - h / 2.0,
                },
                e,
                &c,
            ),
            Self::error(
                &GeometricEntity::Circle {
                    pos: *c1_pos,
                    radius: *c1_radius + h / 2.0,
                },
                e,
                &c,
            ),
        ];

        let x_derivative = (x_errors[1] - x_errors[0]) / h;
        let y_derivative = (y_errors[1] - y_errors[0]) / h;
        let r_derivative = (r_errors[1] - r_errors[0]) / h;
        let step = Vector2::new(x_derivative, y_derivative);
        *c1_pos -= step * step_size;
        *c1_radius -= r_derivative * step_size;
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;

    use super::*;

    #[test]
    fn contraint_possibility_matrix() {
        let point = GeometricEntity::Point {
            pos: Vector2::<f64>::zeros(),
        };
        let circle = GeometricEntity::Circle {
            pos: Vector2::<f64>::zeros(),
            radius: 0.0,
        };
        let line = GeometricEntity::Line {
            offset: Vector2::<f64>::zeros(),
            direction: Vector2::<f64>::zeros(),
        };

        assert!(BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Coincident
        ));
        assert!(BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Horizontal
        ));
        assert!(BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Vertical
        ));
        assert!(!BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Tangent
        ));
        assert!(!BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Parallel
        ));
        assert!(!BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Perpendicular
        ));
        assert!(!BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Colinear
        ));
        assert!(BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Distance { x: 0.0 }
        ));
        assert!(!BiConstraint::possible(
            &point,
            &point,
            &ConstraintType::Angle { x: 0.0 }
        ));
        // --
        assert!(BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Coincident
        ));
        assert!(BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Horizontal
        ));
        assert!(BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Vertical
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Tangent
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Parallel
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Perpendicular
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Colinear
        ));
        assert!(BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Distance { x: 0.0 }
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &point,
            &ConstraintType::Angle { x: 0.0 }
        ));
        // --
        assert!(BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Coincident
        ));
        assert!(BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Horizontal
        ));
        assert!(BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Vertical
        ));
        assert!(BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Tangent
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Parallel
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Perpendicular
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Colinear
        ));
        assert!(BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Distance { x: 0.0 }
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &circle,
            &ConstraintType::Angle { x: 0.0 }
        ));
        // --
        assert!(BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Coincident
        ));
        assert!(!BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Horizontal
        ));
        assert!(!BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Vertical
        ));
        assert!(!BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Tangent
        ));
        assert!(!BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Parallel
        ));
        assert!(!BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Perpendicular
        ));
        assert!(!BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Colinear
        ));
        assert!(BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Distance { x: 0.0 }
        ));
        assert!(!BiConstraint::possible(
            &point,
            &line,
            &ConstraintType::Angle { x: 0.0 }
        ));
        // --
        assert!(BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Coincident
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Horizontal
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Vertical
        ));
        assert!(BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Tangent
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Parallel
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Perpendicular
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Colinear
        ));
        assert!(BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Distance { x: 0.0 }
        ));
        assert!(!BiConstraint::possible(
            &circle,
            &line,
            &ConstraintType::Angle { x: 0.0 }
        ));
        // --
        assert!(!BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Coincident
        ));
        assert!(!BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Horizontal
        ));
        assert!(!BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Vertical
        ));
        assert!(!BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Tangent
        ));
        assert!(BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Parallel
        ));
        assert!(BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Perpendicular
        ));
        assert!(BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Colinear
        ));
        assert!(BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Distance { x: 0.0 }
        ));
        assert!(BiConstraint::possible(
            &line,
            &line,
            &ConstraintType::Angle { x: 0.0 }
        ));
    }
}
