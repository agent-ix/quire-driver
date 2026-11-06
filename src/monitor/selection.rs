// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bind an independent selection to one immutable admitted monitor root.

use quire_contract_model::{
    CheckedMonitorClock, CheckedMonitorClockRead, CheckedMonitorPlanRef, CheckedNodeId,
    CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageV2, CheckedSelectionRole, CheckedTerm,
    ClockConfiguration, LawRole, NativeJoinBudget,
};
use quire_exact::Cancel;

use crate::monitor_refusal::{target_mismatch_at, TargetPremise};

use super::budget::Budget;
use super::{
    MonitorPreparationError as Error, MonitorPreparationRequest, MonitorSelectionField as Field,
};

/// A partial package/root/occurrence/target binding, without clock admission or a program.
///
/// This private-field borrow cannot be rebound to another package instance.
/// It establishes no complete generated-source type environment or admitted
/// clock/order/progress association and authorizes no runtime initialization.
pub struct AdmittedMonitorSelection<'a> {
    pub(super) request: &'a MonitorPreparationRequest<'a>,
    pub(super) plan: CheckedMonitorPlanRef<'a>,
    pub(super) work: u64,
}

impl<'a> AdmittedMonitorSelection<'a> {
    /// Original independently supplied request, immutably borrowed for this selection.
    pub fn request(&self) -> &'a MonitorPreparationRequest<'a> {
        self.request
    }
    /// Exact admitted package instance to which this selection is bound.
    pub fn package(&self) -> &'a CheckedPackageV2 {
        self.request.package
    }
    /// Exact package-borrowed admitted plan whose independent values matched.
    pub fn plan(&self) -> CheckedMonitorPlanRef<'a> {
        self.plan
    }
    /// Operating work admitted while matching the independent selection.
    pub fn work(&self) -> u64 {
        self.work
    }
}

fn mismatch(field: Field, node: &CheckedNodeId) -> Error {
    Error::Selection {
        field,
        node: Box::new(node.clone()),
    }
}

fn matches_configured_clock(
    expected: &ClockConfiguration,
    actual: CheckedMonitorClock<'_>,
) -> bool {
    match actual {
        CheckedMonitorClock::Configured(configured) => expected == configured,
        CheckedMonitorClock::ModelTime { .. } => false,
        CheckedMonitorClock::TraceTime => false,
    }
}

pub(super) fn bind<'a>(
    request: &'a MonitorPreparationRequest<'a>,
    budget: &mut Budget<'_>,
) -> Result<CheckedMonitorPlanRef<'a>, Error> {
    budget.charge(1)?;
    let root = request.selection.root;
    if !std::ptr::eq(request.selection.package, request.package) {
        return Err(mismatch(Field::Package, root));
    }
    budget.inspect(root)?;
    let mut found = None;
    for plan in request.package.monitor_plans() {
        budget.inspect(plan.node_id())?;
        if plan.node_id() == root {
            found = Some(plan);
            break;
        }
    }
    let plan = found.ok_or_else(|| mismatch(Field::Root, root))?;
    budget.inspect(request.package_lock)?;
    budget.inspect(request.package.lock())?;
    if request.package_lock != request.package.lock() {
        return Err(mismatch(Field::PackageLock, root));
    }
    budget.inspect(request.root_key)?;
    budget.inspect(plan.structural_key())?;
    if request.root_key != plan.structural_key() {
        return Err(mismatch(Field::RootKey, root));
    }
    budget.inspect(request.clause)?;
    budget.inspect(plan.clause())?;
    for (expected, entry) in request.entry_keys.into_iter().zip([
        plan.init(),
        plan.on_event(),
        plan.on_watermark(),
        plan.on_gap(),
    ]) {
        budget.inspect(expected)?;
        budget.inspect(entry.structural_key())?;
        if expected != entry.structural_key() {
            return Err(mismatch(Field::EntryKey, entry.node_id()));
        }
    }
    check_clause(request, plan, budget)?;
    budget.inspect(request.profile)?;
    let mut profile_found = false;
    for profile in &request.package.lock().profile_selections {
        budget.inspect(profile)?;
        if profile == request.profile {
            profile_found = true;
            break;
        }
    }
    if !profile_found {
        return Err(mismatch(Field::Profile, plan.clause()));
    }
    check_target(request, plan, budget)?;
    Ok(plan)
}

fn check_clause(
    request: &MonitorPreparationRequest<'_>,
    plan: CheckedMonitorPlanRef<'_>,
    budget: &mut Budget<'_>,
) -> Result<(), Error> {
    let invalid = || Error::InvalidAdmittedClause {
        node: Box::new(plan.clause().clone()),
    };
    let Some(CheckedTerm::Application {
        operation,
        arguments,
        ..
    }) = request.package.term(plan.clause())
    else {
        return Err(invalid());
    };
    let Some(CheckedTerm::Reference { target: subject }) = arguments.first() else {
        return Err(invalid());
    };
    budget.inspect(request.subject)?;
    budget.inspect(subject)?;
    if request.subject != subject {
        return Err(mismatch(Field::Subject, plan.clause()));
    }
    let operation = operation.read().map_err(|_| invalid())?;
    if request.profile.role != CheckedSelectionRole::TemporalProfile {
        return Err(mismatch(Field::Profile, plan.clause()));
    }
    let mut selected_law = false;
    for law in operation.laws {
        budget.inspect(&law.definition)?;
        if law.role == LawRole::TemporalProfile && law.definition == request.profile.definition {
            selected_law = true;
        }
    }
    if !selected_law {
        return Err(mismatch(Field::Profile, plan.clause()));
    }

    Ok(())
}

fn check_target(
    request: &MonitorPreparationRequest<'_>,
    plan: CheckedMonitorPlanRef<'_>,
    budget: &mut Budget<'_>,
) -> Result<(), Error> {
    let refuse = |premise, index| {
        // The independent clause occurrence belongs to the clause, not the
        // generated monitor root. Do not select an arbitrary root occurrence.
        target_mismatch_at(request.package, plan.node_id(), premise, index, None)
            .map(|mismatch| Error::TargetMismatch(Box::new(mismatch)))
            .unwrap_or_else(Error::Internal)
    };
    budget.inspect(request.counter)?;
    budget.inspect(plan.counter())?;
    if request.counter != plan.counter() {
        return Err(refuse(TargetPremise::Counter, None));
    }
    budget.charge(1)?;
    if request.counter_width_bits != plan.counter_width_bits() {
        return Err(refuse(TargetPremise::CounterWidth, None));
    }
    budget.inspect(request.tick_unit)?;
    budget.inspect(plan.tick_unit())?;
    if request.tick_unit != plan.tick_unit() {
        return Err(refuse(TargetPremise::TickUnit, None));
    }
    budget.charge(1)?;
    if request.max_reading_gap != plan.max_reading_gap() {
        return Err(refuse(TargetPremise::MaxReadingGap, None));
    }
    budget.inspect(request.max_event_rate)?;
    budget.inspect(plan.max_event_rate())?;
    if request.max_event_rate != plan.max_event_rate() {
        return Err(refuse(TargetPremise::MaxEventRate, None));
    }
    budget.inspect(request.uncertainty)?;
    budget.inspect(plan.uncertainty())?;
    if request.uncertainty != plan.uncertainty() {
        return Err(refuse(TargetPremise::Uncertainty, None));
    }
    let mut buffers = plan.buffers();
    budget.charge(1)?;
    if buffers.len() != request.buffers.len() {
        return Err(mismatch(Field::BufferAssociation, plan.node_id()));
    }
    for (index, (owner, capacity)) in request.buffers.iter().enumerate() {
        budget.inspect(owner)?;
        budget.inspect(capacity)?;
        let Some(buffer) = buffers.next() else {
            return Err(mismatch(Field::BufferAssociation, plan.node_id()));
        };
        budget.inspect(buffer.subformula())?;
        budget.inspect(buffer.capacity())?;
        if owner != buffer.subformula() {
            return Err(mismatch(Field::BufferAssociation, buffer.subformula()));
        }
        if capacity.as_ref() != buffer.capacity() {
            return Err(refuse(TargetPremise::BufferCapacity, Some(index)));
        }
    }
    budget.inspect(request.activation_capacity)?;
    budget.inspect(plan.activation_capacity())?;
    if request.activation_capacity != plan.activation_capacity() {
        return Err(refuse(TargetPremise::ActivationCapacity, None));
    }
    Ok(())
}

/// Verify independently supplied identities and target premises before lowering.
pub fn admit_monitor_selection<'a>(
    request: &'a MonitorPreparationRequest<'a>,
    cancel: &Cancel,
) -> Result<AdmittedMonitorSelection<'a>, Error> {
    let mut budget = Budget::new(request.limits, cancel);
    select_with_budget(request, &mut budget)
}

pub(super) fn select_with_budget<'a>(
    request: &'a MonitorPreparationRequest<'a>,
    budget: &mut Budget<'_>,
) -> Result<AdmittedMonitorSelection<'a>, Error> {
    let plan = bind(request, budget)?;
    if request.clause != plan.clause() {
        return Err(mismatch(Field::Clause, plan.node_id()));
    }
    budget.inspect(request.occurrence)?;
    if request.occurrence.node_id != *plan.clause()
        || request.occurrence.role != CheckedOccurrenceRole::Claim
    {
        return Err(mismatch(Field::Occurrence, plan.clause()));
    }
    let mut occurrence_found = false;
    for row in request.package.source_map() {
        budget.inspect(row)?;
        if row == request.occurrence {
            occurrence_found = true;
            break;
        }
    }
    if !occurrence_found {
        return Err(mismatch(Field::Occurrence, plan.clause()));
    }
    if let Some(selected_clock) = request.clock_binding {
        budget.inspect(selected_clock)?;
        if selected_clock.declaration != *plan.clause()
            || selected_clock.occurrence != request.occurrence.ordinal
        {
            return Err(mismatch(Field::Clock, plan.clause()));
        }
        let mut clock_found = false;
        for row in request.package.temporal_bindings().unwrap_or_default() {
            budget.inspect(row)?;
            if row == selected_clock {
                clock_found = true;
                break;
            }
        }
        if !clock_found {
            return Err(mismatch(Field::Clock, plan.clause()));
        }
    }

    Ok(AdmittedMonitorSelection {
        request,
        plan,
        work: budget.work(),
    })
}

/// A distinct capability retaining one genuine native admitted clock receipt.
///
/// This establishes no order/progress association, complete source closure,
/// final PlanRoot subject-occurrence mapping or executable program.
pub struct AdmittedMonitorClockSelection<'a> {
    selection: AdmittedMonitorSelection<'a>,
    clock: CheckedMonitorClockRead<'a>,
}

impl<'a> AdmittedMonitorClockSelection<'a> {
    /// The verified selection, including cumulative clock admission work.
    pub fn selection(&self) -> &AdmittedMonitorSelection<'a> {
        &self.selection
    }
    /// The native clock, original claim name and exact admitted occurrence.
    pub fn clock(&self) -> &CheckedMonitorClockRead<'a> {
        &self.clock
    }
}

/// Admit the explicitly selected claim's clock under the cumulative allowance.
///
/// Unlike the partial selection entry, this entry requires the native owner
/// read even when the optional independent clock expectation is absent. A
/// missing clock gives the unchanged native refusal and no clock capability.
/// The existing package/root, subject, profile, target and full source-map
/// validations remain required. No occurrence or clock name is chosen here.
pub fn admit_monitor_clock_selection<'a>(
    request: &'a MonitorPreparationRequest<'a>,
    cancel: &Cancel,
) -> Result<AdmittedMonitorClockSelection<'a>, Error> {
    let mut budget = Budget::new(request.limits, cancel);
    let plan = bind(request, &mut budget)?;
    clock_with_budget(request, plan, &mut budget)
}

pub(super) fn resume_clock<'a>(
    selection: &AdmittedMonitorSelection<'a>,
    cancel: &Cancel,
) -> Result<AdmittedMonitorClockSelection<'a>, Error> {
    let mut budget = Budget::resume(selection, cancel)?;
    clock_with_budget(selection.request, selection.plan, &mut budget)
}

fn clock_with_budget<'a>(
    request: &'a MonitorPreparationRequest<'a>,
    plan: CheckedMonitorPlanRef<'a>,
    budget: &mut Budget<'_>,
) -> Result<AdmittedMonitorClockSelection<'a>, Error> {
    budget.inspect(request.occurrence)?;
    let mut occurrence_found = false;
    for row in request.package.source_map() {
        budget.inspect(row)?;
        if row == request.occurrence {
            occurrence_found = true;
            break;
        }
    }
    budget.charge(0)?;
    let mut native_budget = NativeJoinBudget::new(budget.remaining(), budget.remaining_storage());
    let clock = plan
        .clock_for_claim(
            request.clause,
            &CheckedOccurrence {
                role: request.occurrence.role.clone(),
                ordinal: request.occurrence.ordinal,
            },
            &mut native_budget,
        )
        .map_err(|native| Error::ClockRead(Box::new(native)))?;
    // Native failures carry their own consumed/attempted work unchanged. Only
    // a success continues with the live cumulative driver ledger.
    budget.charge(clock.work())?;
    if request.occurrence.node_id != *clock.clause() || !occurrence_found {
        return Err(mismatch(Field::Occurrence, plan.clause()));
    }
    if let Some(expected) = request.clock_binding {
        budget.inspect(expected)?;
        if expected.declaration != *clock.clause()
            || expected.occurrence != clock.occurrence().ordinal
            || expected.clock_name.as_ref() != clock.clock_name()
            || !matches_configured_clock(&expected.clock, clock.clock())
        {
            return Err(mismatch(Field::Clock, plan.clause()));
        }
    }
    Ok(AdmittedMonitorClockSelection {
        selection: AdmittedMonitorSelection {
            request,
            plan,
            work: budget.work(),
        },
        clock,
    })
}

#[cfg(test)]
mod tests {
    use super::matches_configured_clock;
    use ix_trace_rs::trace;
    use quire_contract_model::{
        CheckedMonitorClock, CheckedNodeId, CheckedNodePathSegment, ClockConfiguration,
    };

    #[trace("FR-015-AC-13")]
    #[test]
    fn configured_clock_expectation_accepts_only_equal_configured_binding() {
        let expected = ClockConfiguration::TimestampedEvent {
            timestamp_unit: "ms".into(),
        };
        let different = ClockConfiguration::TimestampedEvent {
            timestamp_unit: "second".into(),
        };
        assert!(matches_configured_clock(
            &expected,
            CheckedMonitorClock::Configured(&expected)
        ));
        assert!(!matches_configured_clock(
            &expected,
            CheckedMonitorClock::Configured(&different)
        ));
    }

    #[trace("FR-015-AC-13")]
    #[test]
    fn configured_clock_expectation_refuses_model_time_binding() {
        let expected = ClockConfiguration::TimestampedEvent {
            timestamp_unit: "ms".into(),
        };
        let node = CheckedNodeId {
            package: "clock-test".into(),
            path: vec![CheckedNodePathSegment::Name("clock".into())],
            ordinal: 0,
        };
        assert!(!matches_configured_clock(
            &expected,
            CheckedMonitorClock::ModelTime {
                timed_root: &node,
                model: &node,
                source: &node,
                unit: &node,
            }
        ));
    }

    #[trace("FR-015-AC-13")]
    #[test]
    fn configured_clock_expectation_refuses_trace_time_binding() {
        let expected = ClockConfiguration::TimestampedEvent {
            timestamp_unit: "ms".into(),
        };
        assert!(!matches_configured_clock(
            &expected,
            CheckedMonitorClock::TraceTime
        ));
    }
}
