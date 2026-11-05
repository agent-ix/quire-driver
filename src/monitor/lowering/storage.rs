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
        // Reallocation may move the already initialized elements.
        budget.charge(u64::try_from(values.len()).unwrap_or(u64::MAX))?;
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
