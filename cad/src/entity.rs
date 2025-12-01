use std::fmt::Debug;
use std::{error::Error, f64::consts::PI};

use enum_variant_type::EnumVariantType;
use nalgebra::{Rotation2, Vector2};
use serde::{Deserialize, Serialize};

use crate::registry::{RegId, Registry};

#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize, Serialize,
)]
pub struct EntityId(pub u16);

impl RegId for EntityId {
    /// We start at 1 to allow for the usage of 0 as a "null" id
    fn new() -> Self {
        Self(1)
    }

    fn increment(self) -> Self {
        let EntityId(id) = self;
        Self(id + 1)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, EnumVariantType)]
#[evt(derive(Debug, Deserialize, Serialize, Clone, Copy))]
pub enum FundamentalEntity {
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

fn vector_angle(a: Vector2<f64>) -> f64 {
    let angle = f64::atan2(a.y, a.x);
    if angle < 0.0 {
        angle + 2.0 * std::f64::consts::PI
    } else {
        angle
    }
}

impl FundamentalEntity {
    pub fn distance_to_position(&self, target: &Vector2<f64>) -> f64 {
        match self {
            FundamentalEntity::Point { pos } => (pos - target).norm(),
            FundamentalEntity::Line { offset, direction } => {
                let ortho_a = target - project(target, direction);
                let ortho_r = offset - project(offset, direction);
                (ortho_r - ortho_a).norm()
            }
            FundamentalEntity::Circle { pos, radius } => ((target - pos).norm() - radius).abs(),
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

        Some(FundamentalEntity::Circle {
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
    pub e1: EntityId,
    pub e2: EntityId,
    pub c: ConstraintType,
}

impl BiConstraint {
    pub fn new(e1: EntityId, e2: EntityId, c: ConstraintType) -> Self {
        Self { e1, e2, c }
    }

    pub fn possible(e1: &FundamentalEntity, e2: &FundamentalEntity, c: &ConstraintType) -> bool {
        match (e1, e2) {
            (FundamentalEntity::Point { .. }, FundamentalEntity::Point { .. }) => matches!(
                c,
                ConstraintType::Coincident
                    | ConstraintType::Distance { .. }
                    | ConstraintType::Vertical
                    | ConstraintType::Horizontal
            ),
            (FundamentalEntity::Point { .. }, FundamentalEntity::Line { .. }) => matches!(
                c,
                ConstraintType::Coincident | ConstraintType::Distance { .. }
            ),
            (FundamentalEntity::Point { .. }, FundamentalEntity::Circle { .. }) => matches!(
                c,
                ConstraintType::Coincident
                    | ConstraintType::Distance { .. }
                    | ConstraintType::Vertical
                    | ConstraintType::Horizontal
            ),
            (FundamentalEntity::Line { .. }, FundamentalEntity::Line { .. }) => matches!(
                c,
                ConstraintType::Parallel
                    | ConstraintType::Perpendicular
                    | ConstraintType::Colinear
                    | ConstraintType::Distance { .. }
                    | ConstraintType::Angle { .. }
            ),
            (FundamentalEntity::Circle { .. }, FundamentalEntity::Line { .. }) => matches!(
                c,
                ConstraintType::Coincident
                    | ConstraintType::Tangent
                    | ConstraintType::Distance { .. }
            ),
            (FundamentalEntity::Circle { .. }, FundamentalEntity::Circle { .. }) => matches!(
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

    pub fn error(e1: &FundamentalEntity, e2: &FundamentalEntity, c: &ConstraintType) -> f64 {
        if Self::possible(e1, e2, c) {
            match (e1, e2) {
                (FundamentalEntity::Point { .. }, FundamentalEntity::Point { .. }) => {
                    Self::error_pp(e1, e2, *c)
                }
                (FundamentalEntity::Point { .. }, FundamentalEntity::Line { .. }) => {
                    Self::error_pl(e1, e2, *c)
                }
                (FundamentalEntity::Point { .. }, FundamentalEntity::Circle { .. }) => {
                    Self::error_pc(e1, e2, *c)
                }
                (FundamentalEntity::Line { .. }, FundamentalEntity::Line { .. }) => {
                    Self::error_ll(e1, e2, *c)
                }
                (FundamentalEntity::Line { .. }, FundamentalEntity::Circle { .. }) => {
                    Self::error_lc(e1, e2, *c)
                }
                (FundamentalEntity::Circle { .. }, FundamentalEntity::Circle { .. }) => {
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
        e1: &mut FundamentalEntity,
        e2: &FundamentalEntity,
        c: &ConstraintType,
        step_size: f64,
    ) {
        match e1 {
            FundamentalEntity::Point { pos } => Self::apply_grad_error_p(pos, e2, *c, step_size),
            FundamentalEntity::Line { offset, direction } => {
                Self::apply_grad_error_l(offset, direction, e2, *c, step_size)
            }
            FundamentalEntity::Circle { pos, radius } => {
                Self::apply_grad_error_c(pos, radius, e2, *c, step_size)
            }
        }
    }

    fn apply_grad_error_p(
        p1_pos: &mut Vector2<f64>,
        e: &FundamentalEntity,
        c: ConstraintType,
        step_size: f64,
    ) {
        let h = 1e-6;
        let x_errors = [
            Self::error(
                &FundamentalEntity::Point {
                    pos: *p1_pos + Vector2::new(-h / 2.0, 0.0),
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Point {
                    pos: *p1_pos + Vector2::new(h / 2.0, 0.0),
                },
                e,
                &c,
            ),
        ];
        let y_errors = [
            Self::error(
                &FundamentalEntity::Point {
                    pos: *p1_pos + Vector2::new(0.0, -h / 2.0),
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Point {
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
        e: &FundamentalEntity,
        c: ConstraintType,
        step_size: f64,
    ) {
        let h = 1e-4;
        let o_x_errors = [
            Self::error(
                &FundamentalEntity::Line {
                    offset: *l_offset + Vector2::new(-h / 2.0, 0.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Line {
                    offset: *l_offset + Vector2::new(h / 2.0, 0.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
        ];
        let o_y_errors = [
            Self::error(
                &FundamentalEntity::Line {
                    offset: *l_offset + Vector2::new(0.0, -h / 2.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Line {
                    offset: *l_offset + Vector2::new(0.0, h / 2.0),
                    direction: *l_direction,
                },
                e,
                &c,
            ),
        ];
        let d_x_errors = [
            Self::error(
                &FundamentalEntity::Line {
                    offset: *l_offset,
                    direction: *l_direction + Vector2::new(-h / 2.0, 0.0),
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Line {
                    offset: *l_offset,
                    direction: *l_direction + Vector2::new(h / 2.0, 0.0),
                },
                e,
                &c,
            ),
        ];
        let d_y_errors = [
            Self::error(
                &FundamentalEntity::Line {
                    offset: *l_offset,
                    direction: *l_direction + Vector2::new(0.0, -h / 2.0),
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Line {
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
        e: &FundamentalEntity,
        c: ConstraintType,
        step_size: f64,
    ) {
        let h = 1e-6;
        let x_errors = [
            Self::error(
                &FundamentalEntity::Circle {
                    pos: *c1_pos + Vector2::new(-h / 2.0, 0.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Circle {
                    pos: *c1_pos + Vector2::new(h / 2.0, 0.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
        ];
        let y_errors = [
            Self::error(
                &FundamentalEntity::Circle {
                    pos: *c1_pos + Vector2::new(0.0, -h / 2.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Circle {
                    pos: *c1_pos + Vector2::new(0.0, h / 2.0),
                    radius: *c1_radius,
                },
                e,
                &c,
            ),
        ];
        let r_errors = [
            Self::error(
                &FundamentalEntity::Circle {
                    pos: *c1_pos,
                    radius: *c1_radius - h / 2.0,
                },
                e,
                &c,
            ),
            Self::error(
                &FundamentalEntity::Circle {
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, EnumVariantType)]
pub enum GuidedEntity {
    #[evt(skip)]
    Point { id: EntityId },
    /// An infinte line such as the Cartesian axes
    #[evt(skip)]
    Line { id: EntityId },
    #[evt(skip)]
    Circle { id: EntityId },
    /// A finite line between two points
    #[evt(derive(Debug, Deserialize, Serialize, Clone, Copy))]
    CappedLine {
        start: EntityId,
        end: EntityId,
        line: EntityId,
    },
    #[evt(derive(Debug, Deserialize, Serialize, Clone, Copy))]
    ArcThreePoint {
        start: EntityId,
        middle: EntityId,
        end: EntityId,
        circle: EntityId,
    },
}

impl GuidedEntity {
    /// `mouse_pos` is in sketch space
    pub fn filter_selection_attempt(
        &self,
        entity_reg: &Registry<EntityId, FundamentalEntity>,
        mouse_pos: Vector2<f64>,
    ) -> bool {
        match self {
            GuidedEntity::Point { id: _ } => true,
            GuidedEntity::Line { id: _ } => true,
            GuidedEntity::Circle { id: _ } => true,
            GuidedEntity::CappedLine { start, end, line } => {
                if let (
                    Some(FundamentalEntity::Point { pos: start_pos }),
                    Some(FundamentalEntity::Point { pos: end_pos }),
                    Some(FundamentalEntity::Line { .. }),
                ) = (
                    entity_reg.get(start),
                    entity_reg.get(end),
                    entity_reg.get(line),
                ) {
                    let start_point = Point { pos: *start_pos };
                    let end_point = Point { pos: *end_pos };
                    let angle = (end_point.pos - start_point.pos).angle(&Vector2::x());
                    let rot = Rotation2::new(-angle);
                    let start_pos = rot * start_point.pos;
                    let end_pos = rot * end_point.pos;
                    let mouse_pos = rot * mouse_pos;

                    mouse_pos.x >= start_pos.x && mouse_pos.x <= end_pos.x
                        || mouse_pos.x <= start_pos.x && mouse_pos.x >= end_pos.x
                } else {
                    false
                }
            }
            GuidedEntity::ArcThreePoint {
                start,
                middle,
                end,
                circle,
            } => {
                if let (
                    Some(FundamentalEntity::Point { pos: start_pos }),
                    Some(FundamentalEntity::Point { pos: middle_pos }),
                    Some(FundamentalEntity::Point { pos: end_pos }),
                    Some(FundamentalEntity::Circle {
                        pos: circle_pos,
                        radius: circle_radius,
                    }),
                ) = (
                    entity_reg.get(start),
                    entity_reg.get(middle),
                    entity_reg.get(end),
                    entity_reg.get(circle),
                ) {
                    let start_point = Point { pos: *start_pos };
                    let middle_point = Point { pos: *middle_pos };
                    let end_point = Point { pos: *end_pos };
                    let circle_entity = Circle {
                        pos: *circle_pos,
                        radius: *circle_radius,
                    };
                    let tolerance = 5.0 * PI / 180.0;
                    let start_angle = vector_angle(start_point.pos - circle_entity.pos);
                    let mut end_angle = vector_angle(end_point.pos - circle_entity.pos);
                    let middle_angle = vector_angle(middle_point.pos - circle_entity.pos);
                    let mouse_angle = vector_angle(mouse_pos - circle_entity.pos);

                    if middle_angle < start_angle && end_angle > start_angle {
                        end_angle -= 2.0 * PI;
                    }
                    if middle_angle > start_angle && end_angle < start_angle {
                        end_angle += 2.0 * PI;
                    }
                    if middle_angle < end_angle && start_angle > end_angle {
                        end_angle += 2.0 * PI;
                    }
                    if middle_angle > end_angle && start_angle < end_angle {
                        end_angle -= 2.0 * PI;
                    }

                    let min_angle = start_angle.min(end_angle) - tolerance;
                    let max_angle = start_angle.max(end_angle) + tolerance;
                    mouse_angle >= min_angle && mouse_angle <= max_angle
                        || (mouse_angle + 2.0 * PI >= min_angle
                            && mouse_angle + 2.0 * PI <= max_angle)
                        || (mouse_angle - 2.0 * PI >= min_angle
                            && mouse_angle - 2.0 * PI <= max_angle)
                } else {
                    false
                }
            }
        }
    }

    pub fn refers_to(&self, other: EntityId) -> bool {
        match self {
            GuidedEntity::Point { id } => *id == other,
            GuidedEntity::Line { id } => *id == other,
            GuidedEntity::Circle { id } => *id == other,
            GuidedEntity::CappedLine { start, end, line } => {
                *start == other || *end == other || *line == other
            }
            GuidedEntity::ArcThreePoint {
                start,
                middle,
                end,
                circle,
            } => *start == other || *middle == other || *end == other || *circle == other,
        }
    }

    pub fn start_point(self) -> Result<EntityId, Box<dyn Error>> {
        if let Ok(line) = CappedLine::try_from(self) {
            return Ok(line.start);
        }
        if let Ok(arc) = ArcThreePoint::try_from(self) {
            return Ok(arc.start);
        }
        Err("Points, lines and circles don't have any start points".into())
    }

    pub fn end_point(self) -> Result<EntityId, Box<dyn Error>> {
        if let Ok(line) = CappedLine::try_from(self) {
            return Ok(line.end);
        }
        if let Ok(arc) = ArcThreePoint::try_from(self) {
            return Ok(arc.end);
        }
        Err("Points, lines and circles don't have any end points".into())
    }
}

impl CappedLine {
    /// Returns *(p, v)* belonging to a parameterization *r = p + tv*. *t=0* returns the start point of
    /// the line. *t=1* returns the end of the line.
    pub fn parametrize(
        &self,
        f_reg: &Registry<EntityId, FundamentalEntity>,
    ) -> (Vector2<f64>, Vector2<f64>) {
        let start_point: Point = (*f_reg.get(&self.start).unwrap()).try_into().unwrap();
        let end_point: Point = (*f_reg.get(&self.end).unwrap()).try_into().unwrap();
        let start_pos = start_point.pos;
        let end_pos = end_point.pos;
        (start_pos, end_pos - start_pos)
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;

    use super::*;

    #[test]
    fn contraint_possibility_matrix() {
        let point = FundamentalEntity::Point {
            pos: Vector2::<f64>::zeros(),
        };
        let circle = FundamentalEntity::Circle {
            pos: Vector2::<f64>::zeros(),
            radius: 0.0,
        };
        let line = FundamentalEntity::Line {
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
