// SPDX-License-Identifier: AGPL-3.0-or-later
//! Operating accounting for borrowed identity and value inspections.

use std::mem::size_of;

use quire_contract_model::CheckedNodeId;
use quire_exact::Cancel;

use super::{
    material::Material, AdmittedMonitorSelection, MonitorPreparationError, MonitorPreparationLimits,
};

#[cfg(test)]
mod tests;

pub(super) struct Budget<'a> {
    cancel: &'a Cancel,
    limits: MonitorPreparationLimits,
    work: u64,
    storage: u64,
}

impl<'a> Budget<'a> {
    pub(super) fn new(limits: MonitorPreparationLimits, cancel: &'a Cancel) -> Self {
        Self {
            cancel,
            limits,
            work: 0,
            storage: 0,
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
            let before = pending.capacity();
            let slot = size_of::<&CheckedNodeId>();
            self.reserve_storage(slot)?;
            if let Err(cause) = pending.try_reserve_exact(1) {
                self.release_storage(slot)?;
                return Err(MonitorPreparationError::Allocation(cause));
            }
            // An allocator may grant more than the exact request. Never keep
            // a worklist whose reported capacity exceeds the caller limit.
            let extra_slots = pending.capacity() - before - 1;
            if extra_slots != 0 {
                let extra =
                    extra_slots
                        .checked_mul(slot)
                        .ok_or(MonitorPreparationError::Storage {
                            limit: self.limits.storage,
                            consumed: self.storage,
                            next: u64::MAX,
                        })?;
                self.reserve_storage(extra)?;
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

    /// Reserve additional live allocation capacity before allocating.
    pub(super) fn reserve_storage(&mut self, bytes: usize) -> Result<(), MonitorPreparationError> {
        self.charge(0)?;
        let next = u64::try_from(bytes).unwrap_or(u64::MAX);
        if next > self.remaining_storage() {
            return Err(MonitorPreparationError::Storage {
                limit: self.limits.storage,
                consumed: self.storage,
                next,
            });
        }
        self.storage += next;
        Ok(())
    }

    /// Release actual freed capacity. Cleanup never polls cancellation.
    pub(super) fn release_storage(&mut self, bytes: usize) -> Result<(), MonitorPreparationError> {
        let released = u64::try_from(bytes).unwrap_or(u64::MAX);
        let Some(remaining) = self.storage.checked_sub(released) else {
            return Err(MonitorPreparationError::StorageAccounting {
                reserved: self.storage,
                released,
            });
        };
        self.storage = remaining;
        Ok(())
    }

    /// Remaining live storage allowance shared by scratch and retained arenas.
    pub(super) fn remaining_storage(&self) -> u64 {
        self.limits.storage - self.storage
    }
    /// Actual current live logical allocation reservations.
    pub(super) fn storage(&self) -> u64 {
        self.storage
    }

    /// Release this walk's scratch after its vector has actually been dropped.
    pub(super) fn release_scratch(&mut self, before: u64) -> Result<(), MonitorPreparationError> {
        let Some(released) = self.storage.checked_sub(before) else {
            return Err(MonitorPreparationError::StorageAccounting {
                reserved: self.storage,
                released: before,
            });
        };
        let bytes =
            usize::try_from(released).map_err(|_| MonitorPreparationError::StorageAccounting {
                reserved: self.storage,
                released,
            })?;
        self.release_storage(bytes)
    }
}
