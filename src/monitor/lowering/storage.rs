// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fallible, charged driver-owned vector and text storage.

use super::{Budget, Error};

fn overflow(budget: &Budget<'_>) -> Error {
    Error::Storage {
        limit: budget.storage() + budget.remaining_storage(),
        consumed: budget.storage(),
        next: u64::MAX,
    }
}

pub(super) fn push<T>(
    values: &mut Vec<T>,
    value: T,
    budget: &mut Budget<'_>,
    reservation: &mut usize,
) -> Result<(), Error> {
    budget.charge(1)?;
    if values.len() == values.capacity() {
        // Admit relocation before any allocation or storage-ledger change.
        budget.charge(u64::try_from(values.len()).unwrap_or(u64::MAX))?;
        let old = values.capacity();
        let additional = old.max(1);
        let bytes = additional
            .checked_mul(size_of::<T>())
            .ok_or_else(|| overflow(budget))?;
        budget.reserve_storage(bytes)?;
        if let Err(error) = values.try_reserve_exact(additional) {
            budget.release_storage(bytes)?;
            return Err(Error::Allocation(error));
        }
        let actual = (values.capacity() - old)
            .checked_mul(size_of::<T>())
            .ok_or_else(|| overflow(budget))?;
        if actual > bytes {
            budget.reserve_storage(actual - bytes)?;
        }
        *reservation = reservation
            .checked_add(actual)
            .ok_or_else(|| overflow(budget))?;
    }
    values.push(value);
    Ok(())
}

pub(super) fn text(value: &str, budget: &mut Budget<'_>) -> Result<String, Error> {
    budget.inspect(value)?;
    budget.reserve_storage(value.len())?;
    let mut owned = String::new();
    if let Err(error) = owned.try_reserve_exact(value.len()) {
        budget.release_storage(value.len())?;
        return Err(Error::Allocation(error));
    }
    if owned.capacity() > value.len() {
        budget.reserve_storage(owned.capacity() - value.len())?;
    }
    owned.push_str(value);
    Ok(owned)
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use quire_exact::Cancel;

    use super::*;
    use crate::monitor::MonitorPreparationLimits;

    #[trace("FR-017-AC-5")]
    #[test]
    fn growth_work_denial_precedes_allocation_and_preserves_live_storage() {
        let mut values = Vec::with_capacity(4);
        values.extend(0_u64..u64::try_from(values.capacity()).unwrap());
        let before = values.clone();
        let capacity = values.capacity();
        let pointer = values.as_ptr();
        let mut reservation = capacity * size_of::<u64>();
        let before_reservation = reservation;
        let cancel = Cancel::new();
        let mut budget = Budget::new(
            MonitorPreparationLimits {
                work: 1,
                storage: 4096,
            },
            &cancel,
        );
        budget.reserve_storage(reservation).unwrap();
        let before_storage = budget.storage();

        let result = push(&mut values, 99, &mut budget, &mut reservation);
        assert!(
            matches!(result, Err(Error::Work { limit: 1, consumed: 1, next }) if next == u64::try_from(before.len()).unwrap())
        );
        assert_eq!(values, before);
        assert_eq!(values.capacity(), capacity);
        assert_eq!(values.as_ptr(), pointer);
        assert_eq!(reservation, before_reservation);
        assert_eq!(budget.storage(), before_storage);
    }
}
