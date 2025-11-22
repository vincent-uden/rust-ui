use crate::{
    entity::{CappedLine, EntityId, FundamentalEntity, GuidedEntity},
    registry::Registry,
};

/// Represents a sequence of [GuidedEntity]s, specifically
/// [GuidedEntity::CappedLine] and [GuidedEntity::ArcThreePoint] or a single
/// [GuidedEntity::Circle]. Loops are stored in mathematically positive
/// orientation.
#[derive(Debug)]
pub struct Loop {
    pub ids: Vec<EntityId>,
}
