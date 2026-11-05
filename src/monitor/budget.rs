// SPDX-License-Identifier: AGPL-3.0-or-later
//! Operating accounting for borrowed identity and value inspections.

use std::io::{self, Write};

use quire_exact::Cancel;
use serde::Serialize;

use super::{MonitorPreparationError, MonitorPreparationLimits};

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

    pub(super) fn inspect<T: Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), MonitorPreparationError> {
        // Count the existing canonical value's serialized material without a
        // JSON tree or an output buffer. The native writer stops at the first
        // unavailable charge, before the subsequent semantic comparison.
        let mut writer = ChargeWriter {
            budget: self,
            stopped: None,
        };
        let result = serde_json::to_writer(&mut writer, value);
        if let Some(stopped) = writer.stopped {
            return Err(stopped);
        }
        result.map_err(MonitorPreparationError::InternalSerialization)
    }

    pub(super) fn remaining(&self) -> u64 {
        self.limits.work - self.work
    }
    pub(super) fn work(&self) -> u64 {
        self.work
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
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
