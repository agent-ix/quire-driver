// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded copying of supplied native diagnostic axes, without resolution.

use std::{collections::TryReserveError, mem::size_of, sync::Arc};

use quire_contract_model::{CheckedNodeId, CheckedNodePathSegment, CheckedOccurrence};
use quire_exact::{Cancel, CancelCause};

use super::MonitorRefusalLocus;
use crate::monitor::MonitorPreparationLimits;

/// A canonical diagnostic copy and its cumulative work and retained logical bytes.
#[derive(Debug)]
pub struct CopiedMonitorLocus {
    /// Original native diagnostic axes, copied without validating their meaning.
    pub locus: MonitorRefusalLocus,
    /// Cumulative material inspection work.
    pub work: u64,
    /// Retained logical allocation bytes, excluding released scratch.
    pub storage: u64,
}

/// Stable categories for bounded diagnostic copying failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitorLocusCopyErrorCode {
    /// Material work exhausted.
    Work,
    /// Logical storage exhausted or its arithmetic overflowed.
    Storage,
    /// Native fallible allocation failed.
    Allocation,
    /// Native text capacity cannot be boxed without possible allocation.
    Capacity,
    /// Caller cancellation.
    Cancelled,
}

impl MonitorLocusCopyErrorCode {
    /// Stable machine-readable category spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Storage => "storage",
            Self::Allocation => "allocation",
            Self::Capacity => "capacity",
            Self::Cancelled => "cancelled",
        }
    }

    /// Complete stable category catalog.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Work,
            Self::Storage,
            Self::Allocation,
            Self::Capacity,
            Self::Cancelled,
        ]
    }

    /// Decode an exact category spelling; unknown spellings are absent.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

/// Operating failures from bounded diagnostic copying; no diagnostic is allocated on failure.
#[derive(Debug, thiserror::Error)]
pub enum MonitorLocusCopyError {
    /// Work allowance exhausted before inspection.
    #[error("diagnostic copy work exhausted ({consumed} + {next} > {limit})")]
    Work {
        /// Allowance.
        limit: u64,
        /// Consumed work.
        consumed: u64,
        /// Next charge.
        next: u64,
    },
    /// Shared live logical storage allowance exhausted.
    #[error("diagnostic copy storage exhausted ({consumed} + {next} > {limit})")]
    Storage {
        /// Allowance.
        limit: u64,
        /// Current live storage.
        consumed: u64,
        /// Next reservation.
        next: u64,
    },
    /// Original fallible reservation failure.
    #[error("diagnostic copy allocation failed: {0}")]
    Allocation(#[source] TryReserveError),
    /// Text capacity cannot be converted to boxed text without a possible allocation.
    #[error("diagnostic text capacity {actual} differs from requested {requested}")]
    Capacity {
        /// Required exact byte capacity.
        requested: usize,
        /// Allocator-reported capacity.
        actual: usize,
    },
    /// Original caller cancellation cause.
    #[error("diagnostic copy cancelled: {0:?}")]
    Cancelled(CancelCause),
}

impl MonitorLocusCopyError {
    /// Classify this failure without formatting or copying its native cause.
    pub const fn code(&self) -> MonitorLocusCopyErrorCode {
        match self {
            Self::Work { .. } => MonitorLocusCopyErrorCode::Work,
            Self::Storage { .. } => MonitorLocusCopyErrorCode::Storage,
            Self::Allocation(_) => MonitorLocusCopyErrorCode::Allocation,
            Self::Capacity { .. } => MonitorLocusCopyErrorCode::Capacity,
            Self::Cancelled(_) => MonitorLocusCopyErrorCode::Cancelled,
        }
    }
}

type Result<T> = std::result::Result<T, MonitorLocusCopyError>;

struct Ledger<'a> {
    limits: MonitorPreparationLimits,
    cancel: &'a Cancel,
    work: u64,
    storage: u64,
}

impl Ledger<'_> {
    fn charge(&mut self, next: u64) -> Result<()> {
        self.cancel.poll();
        if let Some(cause) = self.cancel.cause() {
            return Err(MonitorLocusCopyError::Cancelled(cause));
        }
        if next > self.limits.work.saturating_sub(self.work) {
            return Err(MonitorLocusCopyError::Work {
                limit: self.limits.work,
                consumed: self.work,
                next,
            });
        }
        self.work += next;
        Ok(())
    }

    fn overflow(&self) -> MonitorLocusCopyError {
        MonitorLocusCopyError::Storage {
            limit: self.limits.storage,
            consumed: self.storage,
            next: u64::MAX,
        }
    }

    fn bytes(&mut self, count: usize, width: usize) -> Result<()> {
        self.charge(0)?;
        let next = count
            .checked_mul(width)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| self.overflow())?;
        if next > self.limits.storage.saturating_sub(self.storage) {
            return Err(MonitorLocusCopyError::Storage {
                limit: self.limits.storage,
                consumed: self.storage,
                next,
            });
        }
        self.storage += next;
        Ok(())
    }

    fn vector<T>(&mut self, count: usize) -> Result<Vec<T>> {
        self.bytes(count, size_of::<T>())?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(MonitorLocusCopyError::Allocation)?;
        self.bytes(output.capacity() - count, size_of::<T>())?;
        Ok(output)
    }

    fn text_work(&mut self, text: &str) -> Result<()> {
        self.charge(1)?;
        self.charge(u64::try_from(text.len()).map_err(|_| self.overflow())?)
    }

    fn text(&mut self, text: &str) -> Result<Box<str>> {
        self.text_work(text)?;
        self.bytes(text.len(), 1)?;
        let mut output = String::new();
        output
            .try_reserve_exact(text.len())
            .map_err(MonitorLocusCopyError::Allocation)?;
        exact_text_capacity(text.len(), output.capacity())?;
        output.push_str(text);
        Ok(output.into_boxed_str())
    }

    fn grow_pending(&mut self, pending: &mut Vec<&CheckedNodeId>) -> Result<()> {
        if pending.len() < pending.capacity() {
            return Ok(());
        }
        // Pay for relocating every live reference before any reservation or mutation.
        self.charge(u64::try_from(pending.len()).map_err(|_| self.overflow())?)?;
        let before = pending.capacity();
        let additional = before.max(1);
        let requested = before
            .checked_add(additional)
            .ok_or_else(|| self.overflow())?;
        self.bytes(additional, size_of::<&CheckedNodeId>())?;
        pending
            .try_reserve_exact(additional)
            .map_err(MonitorLocusCopyError::Allocation)?;
        self.bytes(pending.capacity() - requested, size_of::<&CheckedNodeId>())?;
        Ok(())
    }

    fn shared(&mut self, root: &CheckedNodeId) -> Result<()> {
        let mut pending = self.vector::<&CheckedNodeId>(1)?;
        pending.push(root);
        while let Some(node) = pending.pop() {
            self.charge(1)?;
            self.bytes(1, size_of::<CheckedNodeId>())?;
            self.text_work(&node.package)?;
            self.bytes(node.package.len(), 1)?;
            self.charge(8)?;
            self.charge(1)?;
            self.bytes(node.path.capacity(), size_of::<CheckedNodePathSegment>())?;
            for segment in &node.path {
                self.charge(1)?;
                match segment {
                    CheckedNodePathSegment::Name(name) => {
                        self.text_work(name)?;
                        self.bytes(name.len(), 1)?;
                    }
                    CheckedNodePathSegment::Index(_) | CheckedNodePathSegment::Operand(_) => {
                        self.charge(8)?
                    }
                    CheckedNodePathSegment::Node(child) => {
                        self.charge(1)?;
                        if pending.len() == pending.capacity() {
                            self.grow_pending(&mut pending)?;
                        }
                        pending.push(child);
                    }
                }
            }
        }
        let scratch = pending
            .capacity()
            .checked_mul(size_of::<&CheckedNodeId>())
            .ok_or_else(|| self.overflow())?;
        drop(pending);
        self.storage -= u64::try_from(scratch).map_err(|_| self.overflow())?;
        Ok(())
    }

    fn node(&mut self, node: &CheckedNodeId) -> Result<CheckedNodeId> {
        self.charge(1)?;
        let package = self.text(&node.package)?;
        self.charge(8)?;
        self.charge(1)?;
        let mut path = self.vector(node.path.len())?;
        for segment in &node.path {
            self.charge(1)?;
            path.push(match segment {
                CheckedNodePathSegment::Name(name) => {
                    CheckedNodePathSegment::Name(self.text(name)?)
                }
                CheckedNodePathSegment::Index(index) => {
                    self.charge(8)?;
                    CheckedNodePathSegment::Index(*index)
                }
                CheckedNodePathSegment::Operand(slot) => {
                    self.charge(8)?;
                    CheckedNodePathSegment::Operand(*slot)
                }
                CheckedNodePathSegment::Node(child) => {
                    self.charge(1)?;
                    self.shared(child)?;
                    CheckedNodePathSegment::Node(Arc::clone(child))
                }
            });
        }
        Ok(CheckedNodeId {
            package,
            path,
            ordinal: node.ordinal,
        })
    }
}

fn exact_text_capacity(requested: usize, actual: usize) -> Result<()> {
    if requested != actual {
        return Err(MonitorLocusCopyError::Capacity { requested, actual });
    }
    Ok(())
}

/// Copy already resolved native axes under independent work and live storage limits.
///
/// This makes no admission, membership or path-validity claim. Existing constructors,
/// native operation copying and outer preparation errors remain outside this receipt.
pub fn copy_monitor_locus(
    node: &CheckedNodeId,
    structural_key: &CheckedNodeId,
    body_path: &[usize],
    occurrence: Option<&CheckedOccurrence>,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<CopiedMonitorLocus> {
    let mut ledger = Ledger {
        limits,
        cancel,
        work: 0,
        storage: 0,
    };
    ledger.charge(0)?;
    ledger.charge(1)?; // locus record
    let node = ledger.node(node)?;
    let structural_key = ledger.node(structural_key)?;
    ledger.charge(1)?;
    let mut path = ledger.vector(body_path.len())?;
    for index in body_path {
        ledger.charge(8)?;
        path.push(*index);
    }
    ledger.charge(1)?; // optional occurrence tag
    let occurrence = match occurrence {
        Some(value) => {
            ledger.charge(2)?;
            ledger.charge(8)?;
            Some(CheckedOccurrence {
                role: value.role.clone(),
                ordinal: value.ordinal,
            })
        }
        None => None,
    };
    Ok(CopiedMonitorLocus {
        locus: MonitorRefusalLocus {
            node,
            structural_key,
            body_path: path,
            occurrence,
        },
        work: ledger.work,
        storage: ledger.storage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[test]
    #[trace("FR-016-AC-11")]
    fn capacity_refusal_precedes_box_conversion() {
        assert!(matches!(
            exact_text_capacity(3, 4),
            Err(MonitorLocusCopyError::Capacity {
                requested: 3,
                actual: 4
            })
        ));
        assert!(exact_text_capacity(3, 3).is_ok());
    }

    #[test]
    #[trace("FR-016-AC-10", "FR-016-AC-11")]
    fn overflow_and_native_reservation_failure_are_distinct() {
        let cancel = Cancel::new();
        let mut ledger = Ledger {
            limits: MonitorPreparationLimits {
                work: u64::MAX,
                storage: u64::MAX,
            },
            cancel: &cancel,
            work: 0,
            storage: 0,
        };
        assert!(matches!(
            ledger.bytes(usize::MAX, 2),
            Err(MonitorLocusCopyError::Storage { next: u64::MAX, .. })
        ));
        assert_eq!(ledger.storage, 0);
        // u8 logical charge fits, but Vec's native isize capacity bound refuses.
        assert!(matches!(
            ledger.vector::<u8>(usize::MAX),
            Err(MonitorLocusCopyError::Allocation(_))
        ));
    }
    #[test]
    #[trace("FR-016-AC-10", "FR-016-AC-11")]
    fn denied_frontier_relocation_changes_no_allocation_or_storage() {
        let node = CheckedNodeId {
            package: "".into(),
            path: Vec::new(),
            ordinal: 0,
        };
        let mut pending = vec![&node; 8];
        let pointer = pending.as_ptr();
        let capacity = pending.capacity();
        let cancel = Cancel::new();
        let mut ledger = Ledger {
            limits: MonitorPreparationLimits {
                work: 7,
                storage: u64::MAX,
            },
            cancel: &cancel,
            work: 0,
            storage: 64,
        };
        let error = ledger.grow_pending(&mut pending).unwrap_err();
        assert!(matches!(
            error,
            MonitorLocusCopyError::Work {
                limit: 7,
                consumed: 0,
                next: 8
            }
        ));
        assert_eq!(pending.as_ptr(), pointer);
        assert_eq!(pending.capacity(), capacity);
        assert_eq!(pending.len(), 8);
        assert!(pending.iter().all(|entry| std::ptr::eq(*entry, &node)));
        assert_eq!(ledger.storage, 64);
        assert_eq!(ledger.work, 0);
    }

    #[test]
    #[trace("FR-016-AC-10", "FR-016-AC-11")]
    fn operating_error_catalog_classifies_native_failures_without_formatting() {
        use MonitorLocusCopyErrorCode as Code;
        let cancel = Cancel::new();
        let mut ledger = Ledger {
            limits: MonitorPreparationLimits {
                work: 0,
                storage: 0,
            },
            cancel: &cancel,
            work: 0,
            storage: 0,
        };
        let work = ledger.charge(1).unwrap_err();
        let storage = ledger.bytes(1, 1).unwrap_err();
        ledger.limits = MonitorPreparationLimits {
            work: u64::MAX,
            storage: u64::MAX,
        };
        let allocation = ledger.vector::<u8>(usize::MAX).unwrap_err();
        cancel.cancel(CancelCause::Deadline);
        let cancelled = ledger.charge(0).unwrap_err();
        let errors = [
            (work, Code::Work),
            (storage, Code::Storage),
            (allocation, Code::Allocation),
            (exact_text_capacity(1, 2).unwrap_err(), Code::Capacity),
            (cancelled, Code::Cancelled),
        ];
        assert_eq!(
            Code::all()
                .iter()
                .map(|code| code.as_str())
                .collect::<Vec<_>>(),
            ["work", "storage", "allocation", "capacity", "cancelled"]
        );
        assert_eq!(errors.len(), Code::all().len());
        for (error, expected) in errors {
            assert_eq!(error.code(), expected);
            assert_eq!(Code::from_code(expected.as_str()), Some(expected));
        }
        assert_eq!(Code::from_code("Work"), None);
        assert_eq!(Code::from_code("unknown"), None);
        assert_eq!(Code::from_code(""), None);
    }
}
