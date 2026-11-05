// SPDX-License-Identifier: AGPL-3.0-or-later
//! Operating accounting for borrowed identity and value inspections.

use std::mem::size_of;

use quire_contract_model::CheckedNodeId;
use quire_exact::Cancel;

use super::{
    material::Material, AdmittedMonitorSelection, MonitorPreparationError, MonitorPreparationLimits,
};

pub(super) struct Budget<'a> {
    cancel: &'a Cancel,
    limits: MonitorPreparationLimits,
    work: u64,
}

impl<'a> Budget<'a> {
    pub(super) fn new(limits: MonitorPreparationLimits, cancel: &'a Cancel) -> Self {
        Self {
            cancel,
            limits,
            work: 0,
        }
    }

    /// Resume the same total allowance after admission, without resetting its work prefix.
    pub(super) fn resume(
        selection: &AdmittedMonitorSelection<'_>,
        cancel: &'a Cancel,
    ) -> Result<Self, MonitorPreparationError> {
        let mut budget = Self::new(selection.request().limits, cancel);
        budget.charge(selection.work())?;
        Ok(budget)
    }

    pub(super) fn charge(&mut self, next: u64) -> Result<(), MonitorPreparationError> {
        self.cancel.poll();
        if let Some(cause) = self.cancel.cause() {
            return Err(MonitorPreparationError::Cancelled(cause));
        }
        if next > self.limits.work.saturating_sub(self.work) {
            return Err(MonitorPreparationError::Work {
                limit: self.limits.work,
                consumed: self.work,
                next,
            });
        }
        self.work += next;
        Ok(())
    }

    pub(super) fn inspect<T: Material + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), MonitorPreparationError> {
        value.inspect_material(self)
    }

    pub(super) fn push_key<'key>(
        &mut self,
        pending: &mut Vec<&'key CheckedNodeId>,
        node: &'key CheckedNodeId,
    ) -> Result<(), MonitorPreparationError> {
        self.charge(1)?;
        if pending.len() == pending.capacity() {
            let bytes = pending
                .capacity()
                .checked_mul(size_of::<&CheckedNodeId>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .unwrap_or(u64::MAX);
            let next = u64::try_from(size_of::<&CheckedNodeId>()).unwrap_or(u64::MAX);
            if next > self.limits.storage.saturating_sub(bytes) {
                return Err(MonitorPreparationError::Storage {
                    limit: self.limits.storage,
                    consumed: bytes,
                    next,
                });
            }
            pending
                .try_reserve_exact(1)
                .map_err(MonitorPreparationError::Allocation)?;
            // An allocator may grant more than the exact request. Never keep
            // a worklist whose reported capacity exceeds the caller limit.
            let granted = pending
                .capacity()
                .checked_mul(size_of::<&CheckedNodeId>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .unwrap_or(u64::MAX);
            if granted > self.limits.storage {
                return Err(MonitorPreparationError::Storage {
                    limit: self.limits.storage,
                    consumed: bytes,
                    next: granted.saturating_sub(bytes),
                });
            }
        }
        pending.push(node);
        Ok(())
    }

    pub(super) fn remaining(&self) -> u64 {
        self.limits.work - self.work
    }
    pub(super) fn work(&self) -> u64 {
        self.work
    }
}
