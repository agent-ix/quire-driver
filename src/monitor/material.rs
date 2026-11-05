// SPDX-License-Identifier: AGPL-3.0-or-later
//! Charged borrowed material walks; structural keys never use recursive Serde.

use quire_contract_model::{
    CheckedArtifactRef, CheckedNodeId, CheckedNodePathSegment, CheckedPackageLockV2,
    CheckedRational, CheckedSelection, CheckedSourceMapEntry, ClockConfiguration, ClockExactNumber,
    TemporalBinding,
};

use quire_exact::{NodeKey, PathSegmentView};

use super::{budget::Budget, MonitorPreparationError};

type ReadResult = Result<(), MonitorPreparationError>;

/// The closed set of material inspected by this preparation phase.
///
/// Work pays one unit per record/container/tag, eight per integer scalar and
/// one per UTF-8 byte. The accounting is representation traversal, not a new
/// semantic value, key or serialized wire format.
pub(super) trait Material {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult;
}

impl Material for str {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(1)?;
        budget.charge(u64::try_from(self.len()).unwrap_or(u64::MAX))
    }
}

impl<T: Material + ?Sized> Material for Box<T> {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        self.as_ref().inspect_material(budget)
    }
}

impl Material for CheckedNodeId {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        let mut pending = Vec::new();
        let storage_before = budget.storage();
        let inspected = (|| {
            budget.push_key(&mut pending, self)?;
            while let Some(node) = pending.pop() {
                node.package.inspect_material(budget)?;
                budget.charge(8)?; // exact ordinal
                budget.charge(1)?; // path container
                for segment in &node.path {
                    budget.charge(1)?;
                    match segment {
                        CheckedNodePathSegment::Name(name) => name.inspect_material(budget)?,
                        CheckedNodePathSegment::Index(_) => budget.charge(8)?,
                        CheckedNodePathSegment::Node(child) => {
                            budget.push_key(&mut pending, child)?;
                        }
                    }
                }
            }
            Ok(())
        })();
        drop(pending);
        budget.release_scratch(storage_before)?;
        inspected
    }
}

impl Material for NodeKey {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        let mut pending = Vec::new();
        let storage_before = budget.storage();
        let inspected = (|| {
            budget.push_key(&mut pending, self.view())?;
            while let Some(node) = pending.pop() {
                node.package().inspect_material(budget)?;
                budget.charge(8)?; // exact native ordinal
                budget.charge(1)?; // path container, before borrowed iteration
                for segment in node.path() {
                    budget.charge(1)?;
                    match segment {
                        PathSegmentView::Text(text) => text.inspect_material(budget)?,
                        PathSegmentView::Node(child) => budget.push_key(&mut pending, child)?,
                    }
                }
            }
            Ok(())
        })();
        drop(pending);
        budget.release_scratch(storage_before)?;
        inspected
    }
}

impl Material for CheckedArtifactRef {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(1)?;
        self.authority.inspect_material(budget)?;
        self.identity.inspect_material(budget)
    }
}

impl Material for CheckedSelection {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(2)?; // selection record and native role discriminator
        self.definition.inspect_material(budget)
    }
}

impl Material for CheckedPackageLockV2 {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(2)?; // record and source list
        for source in &self.sources {
            budget.charge(1)?;
            source.authority.inspect_material(budget)?;
            source.identity.inspect_material(budget)?;
        }
        self.edition.inspect_material(budget)?;
        budget.charge(1)?;
        for profile in &self.profile_selections {
            profile.inspect_material(budget)?;
        }
        budget.charge(1)?;
        for definition in &self.definition_selections {
            definition.inspect_material(budget)?;
        }
        budget.charge(1)?;
        for feature in &self.required_features {
            feature.inspect_material(budget)?;
        }
        budget.charge(1)?;
        for dependency in &self.dependency_selections {
            budget.charge(1)?;
            dependency.identity.inspect_material(budget)?;
        }
        budget.charge(1)?; // optional internal-list discriminator
        if let Some(rows) = &self.refinement_internals {
            budget.charge(1)?;
            for row in rows {
                budget.charge(1)?;
                row.refinement.inspect_material(budget)?;
                budget.charge(1)?;
                for class in &row.internal {
                    class.inspect_material(budget)?;
                }
            }
        }
        Ok(())
    }
}

impl Material for CheckedSourceMapEntry {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(1)?;
        self.node_id.inspect_material(budget)?;
        budget.charge(10)?; // role, ordinal and region list
        for region in &self.regions {
            budget.charge(2)?; // region and raw source records
            region.source.authority.inspect_material(budget)?;
            region.source.identity.inspect_material(budget)?;
            budget.charge(16)?; // native start/end scalars
        }
        Ok(())
    }
}

impl Material for CheckedRational {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(1)?;
        self.numerator.inspect_material(budget)?;
        self.denominator.inspect_material(budget)
    }
}

impl Material for ClockExactNumber {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(1)?;
        match self {
            Self::Integer { decimal } => decimal.inspect_material(budget),
            Self::Rational {
                numerator,
                denominator,
            } => {
                numerator.inspect_material(budget)?;
                denominator.inspect_material(budget)
            }
        }
    }
}

impl Material for TemporalBinding {
    fn inspect_material(&self, budget: &mut Budget<'_>) -> ReadResult {
        budget.charge(1)?;
        self.declaration.inspect_material(budget)?;
        budget.charge(8)?;
        self.clock_name.inspect_material(budget)?;
        budget.charge(1)?;
        match &self.clock {
            ClockConfiguration::EventPosition { sequence_authority } => {
                sequence_authority.inspect_material(budget)
            }
            ClockConfiguration::FixedSample {
                epoch,
                period,
                unit,
            } => {
                epoch.inspect_material(budget)?;
                period.inspect_material(budget)?;
                unit.inspect_material(budget)
            }
            ClockConfiguration::TimestampedEvent { timestamp_unit } => {
                timestamp_unit.inspect_material(budget)
            }
        }
    }
}
