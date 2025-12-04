use std::{error::Error, f64::consts::PI};

use enum_variant_type::EnumVariantType;
use nalgebra::{Rotation2, Vector2};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{self, GeoId, GeometricEntity, vector_angle},
    registry::{RegId, Registry},
};

pub type TopoRegistry = Registry<TopoId, TopoEntity>;
impl TopoRegistry {
    pub fn iter_edges(&self) -> impl Iterator<Item = (&TopoId, &Edge)> {
        self.iter()
            .filter_map(|(k, v)| match v {
                TopoEntity::Edge { edge } => Some((k, edge)),
                _ => None,
            })
            .into_iter()
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize, Serialize,
)]
pub struct TopoId(pub u16);
impl RegId for TopoId {
    /// We start at 1 to allow for the usage of 0 as a "null" id
    fn new() -> Self {
        Self(1)
    }

    fn increment(self) -> Self {
        let TopoId(id) = self;
        Self(id + 1)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, EnumVariantType)]
pub enum TopoEntity {
    Point {
        id: GeoId,
    },
    /// An infinte line such as the Cartesian axes
    Line {
        id: GeoId,
    },
    Circle {
        id: GeoId,
    },
    #[evt(skip)]
    /// An entity connecting two points
    Edge {
        edge: Edge,
    },
}

/// An entity connecting two points
#[derive(Debug, Deserialize, Serialize, Clone, Copy, EnumVariantType)]
pub enum Edge {
    /// A finite line between two points
    #[evt(derive(Debug, Deserialize, Serialize, Clone, Copy))]
    CappedLine {
        start: GeoId,
        end: GeoId,
        // TODO: Remove this? Over-constrained system and is probably not needed
        // The points themselves will be constrained to fall onto line
        line: GeoId,
    },
    #[evt(derive(Debug, Deserialize, Serialize, Clone, Copy))]
    ArcThreePoint {
        start: GeoId,
        middle: GeoId,
        end: GeoId,
        circle: GeoId,
    },
}

impl TopoEntity {
    /// `mouse_pos` is in sketch space
    pub fn filter_selection_attempt(
        &self,
        entity_reg: &Registry<GeoId, GeometricEntity>,
        mouse_pos: Vector2<f64>,
    ) -> bool {
        match self {
            TopoEntity::Point { id: _ } => true,
            TopoEntity::Line { id: _ } => true,
            TopoEntity::Circle { id: _ } => true,
            TopoEntity::Edge { edge } => match edge {
                Edge::CappedLine { start, end, line } => {
                    if let (
                        Some(GeometricEntity::Point { pos: start_pos }),
                        Some(GeometricEntity::Point { pos: end_pos }),
                        Some(GeometricEntity::Line { .. }),
                    ) = (
                        entity_reg.get(start),
                        entity_reg.get(end),
                        entity_reg.get(line),
                    ) {
                        let start_point = entity::Point { pos: *start_pos };
                        let end_point = entity::Point { pos: *end_pos };
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
                Edge::ArcThreePoint {
                    start,
                    middle,
                    end,
                    circle,
                } => {
                    if let (
                        Some(GeometricEntity::Point { pos: start_pos }),
                        Some(GeometricEntity::Point { pos: middle_pos }),
                        Some(GeometricEntity::Point { pos: end_pos }),
                        Some(GeometricEntity::Circle {
                            pos: circle_pos,
                            radius: circle_radius,
                        }),
                    ) = (
                        entity_reg.get(start),
                        entity_reg.get(middle),
                        entity_reg.get(end),
                        entity_reg.get(circle),
                    ) {
                        let start_point = entity::Point { pos: *start_pos };
                        let middle_point = entity::Point { pos: *middle_pos };
                        let end_point = entity::Point { pos: *end_pos };
                        let circle_entity = entity::Circle {
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
            },
        }
    }
}

impl<T> From<T> for TopoEntity
where
    T: Into<Edge>,
{
    fn from(value: T) -> Self {
        Self::Edge { edge: value.into() }
    }
}

impl TryInto<Edge> for TopoEntity {
    type Error = String;

    fn try_into(self) -> Result<Edge, Self::Error> {
        match self {
            TopoEntity::Edge { edge } => Ok(edge),
            _ => Err("TopoEntity is not an edge".into()),
        }
    }
}

impl Edge {
    pub fn start_point(self) -> Result<GeoId, Box<dyn Error>> {
        if let Ok(line) = CappedLine::try_from(self) {
            return Ok(line.start);
        }
        if let Ok(arc) = ArcThreePoint::try_from(self) {
            return Ok(arc.start);
        }
        Err("Points, lines and circles don't have any start points".into())
    }

    pub fn end_point(self) -> Result<GeoId, Box<dyn Error>> {
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
        f_reg: &Registry<GeoId, GeometricEntity>,
    ) -> (Vector2<f64>, Vector2<f64>) {
        let start_point: entity::Point = f_reg[self.start].try_into().unwrap();
        let end_point: entity::Point = f_reg[self.end].try_into().unwrap();
        let start_pos = start_point.pos;
        let end_pos = end_point.pos;
        (start_pos, end_pos - start_pos)
    }
}

impl TryFrom<TopoEntity> for CappedLine {
    type Error = String;

    fn try_from(value: TopoEntity) -> Result<Self, Self::Error> {
        match value {
            TopoEntity::Edge { edge } => {
                CappedLine::try_from(edge).map_err(|e| format!("{:?} is not a CappedLine", e))
            }
            _ => Err("Not an edge".into()),
        }
    }
}

impl TryFrom<TopoEntity> for ArcThreePoint {
    type Error = String;

    fn try_from(value: TopoEntity) -> Result<Self, Self::Error> {
        match value {
            TopoEntity::Edge { edge } => {
                ArcThreePoint::try_from(edge).map_err(|e| format!("{:?} is not a ArcThreePoint", e))
            }
            _ => Err("Not an edge".into()),
        }
    }
}

/// Represents a sequence of [GuidedEntity]s, specifically
/// [GuidedEntity::CappedLine] and [GuidedEntity::ArcThreePoint] or a single
/// [GuidedEntity::Circle]. Loops are stored in mathematically positive
/// orientation.
///
/// Can be open or closed.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wire {
    pub ids: Vec<TopoId>,
}

impl Wire {
    pub fn try_into(self, reg: &Registry<TopoId, TopoEntity>) -> Result<Loop, Box<dyn Error>> {
        if self.ids.is_empty() {
            return Err("Wire must contain at least one entity".into());
        }
        let first_guided: Edge = reg[*self.ids.first().unwrap()].try_into()?;
        let last_guided: Edge = reg[*self.ids.last().unwrap()].try_into()?;
        let first = first_guided.start_point()?;
        let last = last_guided.end_point()?;

        if first == last {
            Ok(Loop {
                ids: self.ids.clone(),
            })
        } else {
            Err(format!("{:?} != {:?}, wire is not closed", first, last).into())
        }
    }
}

/// A closed [Wire]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Loop {
    pub ids: Vec<TopoId>,
}

/// An intersection of two parametrized edges
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParametrizedIntersection {
    /// The geometrical point in space
    pub point: entity::Point,
    /// Coordinate of the intersection along the first curve
    pub t: f64,
    /// Coordinate of the intersection along the second curve
    pub s: f64,
}
