// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native copying mechanics; these public structs assert no admission evidence.

use std::{mem::size_of, sync::Arc};

use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodePathSegment as Segment, CheckedOccurrence, CheckedOccurrenceRole,
};
use quire_driver::{
    monitor::MonitorPreparationLimits,
    monitor_refusal::{copy_monitor_locus, MonitorLocusCopyError},
};
use quire_exact::{Cancel, CancelCause};

#[test]
#[trace("FR-016-AC-9", "FR-016-AC-10")]
fn native_axes_share_backing_and_obey_independent_exact_limits() {
    let child = Arc::new(CheckedNodeId {
        package: "é".into(),
        path: Vec::new(),
        ordinal: 7,
    });
    let node = CheckedNodeId {
        package: "p".into(),
        path: vec![
            Segment::Name("µ".into()),
            Segment::Node(Arc::clone(&child)),
            Segment::Node(Arc::clone(&child)),
            Segment::Index(9),
        ],
        ordinal: 2,
    };
    let path = vec![0; 128];
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Generated,
        ordinal: 19,
    };
    // Each root: record, text container+byte, ordinal, path container,
    // name tag+text container+2 bytes, two tag+edge+13-work leaves, index tag+scalar.
    let work = 2 * 55 + 1 + 1 + 8 * 128 + 1 + 2 + 8;
    let storage = 2 * (1 + 4 * size_of::<Segment>() + 2 + 2 * (size_of::<CheckedNodeId>() + 2))
        + 128 * size_of::<usize>();
    let limits = MonitorPreparationLimits {
        work,
        storage: u64::try_from(storage).unwrap(),
    };
    let copied = copy_monitor_locus(
        &node,
        &node,
        &path,
        Some(&occurrence),
        limits,
        &Cancel::new(),
    )
    .unwrap();
    assert_eq!(copied.work, work);
    assert_eq!(copied.storage, limits.storage);
    assert_eq!(copied.locus.node, node);
    assert_eq!(copied.locus.structural_key, node);
    assert_ne!(copied.locus.node.package.as_ptr(), node.package.as_ptr());
    assert_ne!(copied.locus.node.path.as_ptr(), node.path.as_ptr());
    let Segment::Node(retained) = &copied.locus.node.path[1] else {
        panic!("missing native shared edge")
    };
    assert!(Arc::ptr_eq(retained, &child));
    assert!(matches!(
        copy_monitor_locus(
            &node,
            &node,
            &path,
            Some(&occurrence),
            MonitorPreparationLimits {
                work: work - 1,
                ..limits
            },
            &Cancel::new()
        ),
        Err(MonitorLocusCopyError::Work { .. })
    ));
    assert!(matches!(
        copy_monitor_locus(
            &node,
            &node,
            &path,
            Some(&occurrence),
            MonitorPreparationLimits {
                storage: limits.storage - 1,
                ..limits
            },
            &Cancel::new()
        ),
        Err(MonitorLocusCopyError::Storage { .. })
    ));
    let none = copy_monitor_locus(&node, &node, &path, None, limits, &Cancel::new()).unwrap();
    assert_eq!(none.locus.occurrence, None);
    drop((node, child, path, occurrence));
    assert_eq!(copied.locus.body_path, vec![0; 128]);
    assert_eq!(copied.locus.occurrence.unwrap().ordinal, 19);
    let Segment::Node(retained) = &copied.locus.structural_key.path[2] else {
        panic!("missing retained child")
    };
    assert_eq!(retained.package.as_ref(), "é");
}

#[test]
#[trace("FR-016-AC-10")]
fn cancellation_wins_before_work_and_storage_reservations() {
    let node = CheckedNodeId {
        package: "unresolved".into(),
        path: Vec::new(),
        ordinal: 0,
    };
    let cancel = Cancel::new();
    cancel.cancel(CancelCause::Deadline);
    assert!(matches!(
        copy_monitor_locus(
            &node,
            &node,
            &[],
            None,
            MonitorPreparationLimits {
                work: 0,
                storage: 0
            },
            &cancel
        ),
        Err(MonitorLocusCopyError::Cancelled(CancelCause::Deadline))
    ));
}
