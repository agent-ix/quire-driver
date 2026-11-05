// SPDX-License-Identifier: AGPL-3.0-or-later
//! Live storage accounting across retained allocations and key scratch.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, OnceLock,
};

use quire_contract_model::{CheckedNodeId, CheckedNodePathSegment};
use quire_exact::CancelCause;

use super::*;

fn key() -> CheckedNodeId {
    CheckedNodeId {
        package: "independent-key".into(),
        path: Vec::new(),
        ordinal: 0,
    }
}

fn retain_arena(budget: &mut Budget<'_>, bytes: usize) -> Vec<u8> {
    budget.reserve_storage(bytes).unwrap();
    let mut arena = Vec::new();
    arena.try_reserve_exact(bytes).unwrap();
    budget.reserve_storage(arena.capacity() - bytes).unwrap();
    arena
}

#[test]
fn retained_arena_and_key_scratch_share_one_live_limit() {
    let cancel = Cancel::new();
    let slot = size_of::<&CheckedNodeId>();
    let mut budget = Budget::new(
        MonitorPreparationLimits {
            work: u64::MAX,
            storage: 64,
        },
        &cancel,
    );
    let arena = retain_arena(&mut budget, 32);
    let retained = budget.storage();
    // Leave precisely one worklist slot alongside an actual retained allocation.
    budget.limits.storage = retained + u64::try_from(slot).unwrap();
    let mut root = key();
    root.path = (0..3)
        .map(|_| CheckedNodePathSegment::Node(Arc::new(key())))
        .collect();
    assert!(matches!(
        budget.inspect(&root),
        Err(MonitorPreparationError::Storage { consumed, next, .. })
            if consumed == retained + u64::try_from(slot).unwrap()
                && next == u64::try_from(slot).unwrap()
    ));
    assert_eq!(budget.storage(), retained);
    assert_eq!(budget.remaining_storage(), u64::try_from(slot).unwrap());
    // Released traversal scratch is reusable; repeated walks do not accumulate it.
    budget.inspect(&key()).unwrap();
    budget.inspect(&key()).unwrap();
    assert_eq!(budget.storage(), retained);
    let bytes = arena.capacity();
    drop(arena);
    budget.release_storage(bytes).unwrap();
    assert_eq!(budget.storage(), 0);
}

#[test]
fn key_work_failure_releases_scratch_and_preserves_retained_storage() {
    let cancel = Cancel::new();
    let mut budget = Budget::new(
        MonitorPreparationLimits {
            work: 2,
            storage: 64,
        },
        &cancel,
    );
    let arena = retain_arena(&mut budget, 32);
    let retained = budget.storage();
    assert!(matches!(
        budget.inspect(&key()),
        Err(MonitorPreparationError::Work { consumed: 2, .. })
    ));
    assert_eq!(budget.storage(), retained);
    assert_eq!(budget.work(), 2);
    assert_eq!(budget.remaining(), 0);
    let bytes = arena.capacity();
    drop(arena);
    budget.release_storage(bytes).unwrap();
}

#[ix_trace_rs::trace("FR-015-AC-15")]
#[test]
fn cancellation_during_key_walk_releases_scratch_with_original_cause() {
    for native in [false, true] {
        let holder = Arc::new(OnceLock::<Cancel>::new());
        let observer_holder = Arc::downgrade(&holder);
        let polls = Arc::new(AtomicUsize::new(0));
        let observer_polls = Arc::clone(&polls);
        let cancel = Cancel::observing(move || {
            if observer_polls.fetch_add(1, Ordering::Relaxed) == 2 {
                observer_holder
                    .upgrade()
                    .unwrap()
                    .get()
                    .unwrap()
                    .cancel(CancelCause::Deadline);
            }
        });
        holder.set(cancel.clone()).unwrap();
        let mut budget = Budget::new(
            MonitorPreparationLimits {
                work: u64::MAX,
                storage: 64,
            },
            &cancel,
        );
        let arena = retain_arena(&mut budget, 32);
        let retained = budget.storage();
        polls.store(0, Ordering::Relaxed);
        assert!(matches!(
            if native {
                budget.inspect(&native_key())
            } else {
                budget.inspect(&key())
            },
            Err(MonitorPreparationError::Cancelled(CancelCause::Deadline))
        ));
        assert_eq!(budget.work(), 1); // Root pushed, scratch allocated, then cancelled.
        assert_eq!(budget.storage(), retained);
        let bytes = arena.capacity();
        drop(arena);
        budget.release_storage(bytes).unwrap(); // Cleanup must not poll cancellation.
        assert_eq!(budget.storage(), 0);
    }
}

#[test]
fn denied_reservations_and_invalid_releases_preserve_live_counts() {
    let cancel = Cancel::new();
    let mut budget = Budget::new(
        MonitorPreparationLimits {
            work: 10,
            storage: 32,
        },
        &cancel,
    );
    let arena = retain_arena(&mut budget, 32);
    assert!(matches!(
        budget.reserve_storage(1),
        Err(MonitorPreparationError::Storage {
            limit: 32,
            consumed: 32,
            next: 1,
        })
    ));
    assert!(matches!(
        budget.release_storage(33),
        Err(MonitorPreparationError::StorageAccounting {
            reserved: 32,
            released: 33,
        })
    ));
    assert_eq!(budget.storage(), 32);
    cancel.cancel(CancelCause::Requested);
    assert!(matches!(
        budget.reserve_storage(0),
        Err(MonitorPreparationError::Cancelled(CancelCause::Requested))
    ));
    assert_eq!(budget.storage(), 32);
    let bytes = arena.capacity();
    drop(arena);
    budget.release_storage(bytes).unwrap();
    assert_eq!(budget.storage(), 0);
}

fn native_key() -> quire_exact::NodeKey {
    use quire_exact::{NodeKey, PathSegment};
    let child = NodeKey::try_new(
        "child".to_owned(),
        vec![PathSegment::Text("β".to_owned())],
        3,
    )
    .unwrap();
    NodeKey::try_new(
        "root".to_owned(),
        vec![PathSegment::Text("r".to_owned()), PathSegment::Node(child)],
        7,
    )
    .unwrap()
}

#[ix_trace_rs::trace("FR-015-AC-15")]
#[test]
fn native_key_borrowed_material_charges_nested_utf8_and_releases_all_scratch() {
    let key = native_key();
    let cancel = Cancel::new();
    let mut exact = Budget::new(
        MonitorPreparationLimits {
            work: 39,
            storage: 64,
        },
        &cancel,
    );
    exact.inspect(&key).unwrap();
    assert_eq!(exact.work(), 39);
    assert_eq!(exact.storage(), 0);
    let mut one_under = Budget::new(
        MonitorPreparationLimits {
            work: 38,
            storage: 64,
        },
        &cancel,
    );
    assert!(matches!(
        one_under.inspect(&key),
        Err(MonitorPreparationError::Work {
            limit: 38,
            consumed: 37,
            next: 2,
        })
    ));
    assert_eq!(one_under.storage(), 0);
    let mut denied = Budget::new(
        MonitorPreparationLimits {
            work: 39,
            storage: 0,
        },
        &cancel,
    );
    assert!(matches!(
        denied.inspect(&key),
        Err(MonitorPreparationError::Storage { consumed: 0, .. })
    ));
    assert_eq!(denied.storage(), 0);
}
