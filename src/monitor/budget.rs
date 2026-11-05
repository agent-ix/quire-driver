// SPDX-License-Identifier: AGPL-3.0-or-later
//! Operating accounting for borrowed identity and value inspections.

use std::{fmt, io::{self, Write}};

use quire_exact::Cancel;
use serde::Serialize;

use super::{MonitorPreparationError, MonitorPreparationLimits};

pub(super) struct Budget<'a> {
    cancel: &'a Cancel,
    limits: MonitorPreparationLimits,
    work: u64,
    storage: u64,
}

impl<'a> Budget<'a> {
    pub(super) fn new(limits: MonitorPreparationLimits, cancel: &'a Cancel) -> Self {
        Self { cancel, limits, work: 0, storage: 0 }
    }

    pub(super) fn charge(&mut self, next: u64) -> Result<(), MonitorPreparationError> {
        self.cancel.poll();
        if let Some(cause) = self.cancel.cause() {
            return Err(MonitorPreparationError::Cancelled(cause));
        }
        if next > self.limits.work.saturating_sub(self.work) {
            return Err(MonitorPreparationError::Work {
                limit: self.limits.work, consumed: self.work, next,
            });
        }
        self.work += next;
        Ok(())
    }

    pub(super) fn inspect<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), MonitorPreparationError> {
        // Count the existing canonical value's serialized material without a
        // JSON tree or an output buffer. The native writer stops at the first
        // unavailable charge, before the subsequent semantic comparison.
        let mut writer = ChargeWriter { budget: self, stopped: None };
        let result = serde_json::to_writer(&mut writer, value);
        if let Some(stopped) = writer.stopped { return Err(stopped); }
        result.map_err(MonitorPreparationError::InternalSerialization)
    }

    pub(super) fn reserve(&mut self, bytes: usize) -> Result<(), MonitorPreparationError> {
        self.charge(0)?;
        let next = u64::try_from(bytes).unwrap_or(u64::MAX);
        if next > self.limits.storage.saturating_sub(self.storage) {
            return Err(MonitorPreparationError::Storage {
                limit: self.limits.storage, consumed: self.storage, next,
            });
        }
        self.storage += next;
        Ok(())
    }

    pub(super) fn remaining(&self) -> u64 { self.limits.work - self.work }
    pub(super) fn work(&self) -> u64 { self.work }

    pub(super) fn inspect_debug(&mut self, value: &impl fmt::Debug) -> Result<(), MonitorPreparationError> {
        let mut writer = DebugChargeWriter { budget: self, stopped: None };
        let result = fmt::write(&mut writer, format_args!("{value:?}"));
        if let Some(stopped) = writer.stopped { return Err(stopped); }
        // The only fallible writer operation records its exact operating stop.
        debug_assert!(result.is_ok());
        Ok(())
    }
}

struct DebugChargeWriter<'budget, 'cancel> {
    budget: &'budget mut Budget<'cancel>,
    stopped: Option<MonitorPreparationError>,
}

impl fmt::Write for DebugChargeWriter<'_, '_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        match self.budget.charge(u64::try_from(value.len()).unwrap_or(u64::MAX)) {
            Ok(()) => Ok(()),
            Err(stopped) => { self.stopped = Some(stopped); Err(fmt::Error) }
        }
    }
}

struct ChargeWriter<'budget, 'cancel> {
    budget: &'budget mut Budget<'cancel>,
    stopped: Option<MonitorPreparationError>,
}

impl Write for ChargeWriter<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let amount = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        match self.budget.charge(amount) {
            Ok(()) => Ok(bytes.len()),
            Err(stopped) => {
                self.stopped = Some(stopped);
                Err(io::Error::other("operating inspection stopped"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
