use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{
    entity::{ArcThreePoint, CappedLine, EntityId, FundamentalEntity, GuidedEntity},
    registry::Registry,
};

/// Represents a sequence of [GuidedEntity]s, specifically
/// [GuidedEntity::CappedLine] and [GuidedEntity::ArcThreePoint] or a single
/// [GuidedEntity::Circle]. Loops are stored in mathematically positive
/// orientation.
///
/// Can be open or closed.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wire {
    pub ids: Vec<EntityId>,
}

impl Wire {
    pub fn try_into(self, reg: &Registry<EntityId, GuidedEntity>) -> Result<Loop, Box<dyn Error>> {
        if self.ids.len() < 2 {
            return Err("Wire must be at least 2 entities long".into());
        }
        let first_guided = reg
            .get(self.ids.first().unwrap())
            .ok_or("First id not present in registry")?;
        let last_guided = reg
            .get(self.ids.last().unwrap())
            .ok_or("Last id not present in registry")?;
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
    pub ids: Vec<EntityId>,
}
