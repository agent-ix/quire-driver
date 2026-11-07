// SPDX-License-Identifier: AGPL-3.0-or-later
//! Same-emission producer rows through the driver's atomic IR closure handoff.

use std::sync::LazyLock;

use ix_trace_rs::trace;
use quire_contract_model::{NativeJoinBudget, SelectedMonitorTypeClosureErrorCode};
use quire_driver::monitor::{
    admit_monitor_final_subject, read_monitor_type_closure, MonitorPreparationError,
    MonitorPreparationRequest,
};
use quire_exact::Cancel;

#[path = "common/monitor.rs"]
mod monitor_fixture;
use monitor_fixture::{fixture, read_limits, Fixture, DISTINCT_CLAUSE};

static FIXTURE: LazyLock<Fixture> =
    LazyLock::new(|| fixture("SelectedMonitor", "counter", DISTINCT_CLAUSE));

fn with_request(test: impl FnOnce(MonitorPreparationRequest<'_>)) {
    monitor_fixture::with_request(
        &FIXTURE,
        FIXTURE.subject.occurrence().origin().ordinal(),
        test,
    );
}

#[trace("FR-015-AC-14")]
#[test]
fn authentic_same_emission_closure_retains_all_native_fact_channels() {
    with_request(|request| {
        let selected = admit_monitor_final_subject(
            &request,
            &FIXTURE.subject,
            &FIXTURE.emission,
            &FIXTURE.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .unwrap();
        let mut budget = NativeJoinBudget::new(1_000_000, 1_000_000);
        let closure = read_monitor_type_closure(&selected, &mut budget, &Cancel::new()).unwrap();
        assert_eq!(closure.plan().node_id(), request.selection_root());
        assert!(!closure.exact_types().is_empty());
        assert!(!closure.clocks().is_empty());
        assert!(!closure.model_types().records().is_empty());
        assert!(budget.consumed().work >= closure.work());
        assert!(closure.work() > 0);
    });
}

#[trace("FR-015-AC-14")]
#[test]
fn closure_budget_boundary_keeps_native_typed_stop_and_no_owner() {
    with_request(|request| {
        let selected = admit_monitor_final_subject(
            &request,
            &FIXTURE.subject,
            &FIXTURE.emission,
            &FIXTURE.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .unwrap();
        let mut measured = NativeJoinBudget::new(1_000_000, 1_000_000);
        let _ = read_monitor_type_closure(&selected, &mut measured, &Cancel::new()).unwrap();
        let exact_work = measured.consumed().work;
        let exact_storage = measured.consumed().storage;
        let mut exact = NativeJoinBudget::new(exact_work, exact_storage);
        assert!(read_monitor_type_closure(&selected, &mut exact, &Cancel::new()).is_ok());
        let mut short = NativeJoinBudget::new(exact_work - 1, exact_storage);
        let error = read_monitor_type_closure(&selected, &mut short, &Cancel::new()).unwrap_err();
        assert!(matches!(
            error,
            MonitorPreparationError::ClosureRead(native)
                if native.code == SelectedMonitorTypeClosureErrorCode::WorkLimit
                    && native.denied.is_some()
                    && native.denied_locus.is_some()
        ));
    });
}
