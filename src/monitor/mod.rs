// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded monitor selection and preparation from admitted package facts.
//!
//! The implemented selection phase does not initialize a runtime or substitute
//! a selected-fact projection for a complete executable declaration closure.

mod budget;
mod error;
mod request;
mod selection;
mod projection;

pub use error::{MonitorPreparationError, MonitorPreparationErrorCode, MonitorSelectionField};
pub use request::{MonitorPreparationLimits, MonitorPreparationRequest, MonitorSelection};
pub use selection::{admit_monitor_selection, AdmittedMonitorSelection};
pub use projection::project_monitor_initializer;
