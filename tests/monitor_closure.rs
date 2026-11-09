// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plan-authority admission and selected type closure from a genuine PlanRoot.

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{PlanAuthoritySelection, PlanAuthorityValidationCaps};
use quire_driver::monitor::{
    collect_monitor_closure_rows, MonitorPreparationErrorCode, MonitorPreparationLimits,
};
use quire_exact::Cancel;

/// Native PlanRoot support shared by monitor integration suites.
#[path = "common/monitor.rs"]
pub mod monitor_fixture;
/// Genuine observation-owner progress admission.
#[path = "common/observation.rs"]
pub mod observation;

fn caps() -> PlanAuthorityValidationCaps {
    PlanAuthorityValidationCaps {
        read: monitor_fixture::read_limits(),
        order: quire_canonical::Limits::new(1 << 20),
    }
}

fn collect(
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<quire_driver::monitor::MonitorClosureRows, quire_driver::monitor::MonitorPreparationError>
{
    let fixture = monitor_fixture::fixture("SelectedMonitor", "counter", "");
    let root = fixture.into_root();
    let (progress, subject) = observation::admitted(Some(vec![observation::row(&root)]));
    let source = root.checked_unit().source().identity().clone();
    let claim = root.plan().subject().clone();
    let clock = observation::clock();
    let observation_source = observation::source();
    collect_monitor_closure_rows(
        root,
        PlanAuthoritySelection {
            source: &source,
            claim: &claim,
            observation_subject: &subject,
            observation_clock: &clock,
            observation_source: &observation_source,
        },
        Some(progress),
        caps(),
        limits,
        cancel,
    )
}

const LIMITS: MonitorPreparationLimits = MonitorPreparationLimits {
    work: 100_000_000,
    storage: 100_000_000,
};

#[trace("TC-033", "FR-019-AC-1")]
#[test]
fn genuine_plan_root_admits_authority_and_reads_the_selected_closure() {
    let cancel = Cancel::new();
    let mut rows = collect(LIMITS, &cancel).expect("genuine root admits");
    let before = rows.budget().consumed();
    assert!(before.work > 0);
    let work = rows
        .read_closure(&cancel, |authority, closure| {
            assert_eq!(
                closure.authority().order_value(),
                authority.admitted_order()
            );
            assert_eq!(
                closure.authority().progress_association(),
                authority.progress_context()
            );
            assert!(!closure.exact_types().is_empty());
            closure.work()
        })
        .expect("closure reads");
    assert!(work > 0);
    assert!(rows.budget().consumed().work > before.work);
}

#[trace("TC-033", "FR-019-AC-3")]
#[test]
fn a_starved_budget_stops_with_a_work_or_storage_code() {
    let cancel = Cancel::new();
    let error = collect(
        MonitorPreparationLimits {
            work: 1,
            storage: 1,
        },
        &cancel,
    )
    .err()
    .expect("starved");
    assert!(matches!(
        error.code(),
        MonitorPreparationErrorCode::Work | MonitorPreparationErrorCode::Storage
    ));
}

#[trace("TC-033", "FR-019-AC-5")]
#[test]
fn cancellation_before_admission_returns_cancelled() {
    let cancel = Cancel::new();
    cancel.cancel(quire_exact::CancelCause::Requested);
    let error = collect(LIMITS, &cancel).err().expect("cancelled");
    assert_eq!(error.code(), MonitorPreparationErrorCode::Cancelled);
}
