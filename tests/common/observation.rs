// SPDX-License-Identifier: AGPL-3.0-or-later
//! Genuine observation-owner admission and progress association rows.
// Copied from the QSL authority tests so the progress view is the owner's real one.
#![allow(dead_code, missing_docs)]

use quire_observation::authority::{
    self, clock, progress, AuthoritySelection, Context, History, Limits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, ProducerConfiguration,
    QualifiedSubject, ResourceLimits, Revision, ScopeKind, ScopeSelection, StaticProducerBundle,
    SubjectIdentity, SubjectKind, ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT,
    PRODUCER_INTERFACE_VERSION,
};

pub fn id(value: &str) -> Identity {
    Identity::new(value)
}

pub fn clock() -> clock::Selection {
    clock::Selection::new(id("clock:monitor-events"), id("1"))
}

pub fn source() -> Identity {
    id("source:monitor-events")
}

pub fn admitted(rows: Option<Vec<progress::AssociationRow>>) -> (progress::View, SubjectSelection) {
    admitted_with_progress_limits(rows, Limits::owner_max()).expect("real progress owner admission")
}

pub fn admitted_with_progress_limits(
    rows: Option<Vec<progress::AssociationRow>>,
    progress_limits: Limits,
) -> Result<(progress::View, SubjectSelection), authority::Error> {
    let limits = Limits::owner_max();
    let mut candidate = StaticProducerBundle {
        bundle_identity: Some("bundle:monitor-observations".to_owned()),
        bundle_revision: Some(Revision::producer("1")),
        interface_version: Some(PRODUCER_INTERFACE_VERSION.to_owned()),
        configuration: Some(ProducerConfiguration {
            configuration_identity: "configuration:monitor-observations".to_owned(),
            digest: StaticProducerBundle::default()
                .canonical_digest_selection()
                .unwrap(),
        }),
        ..StaticProducerBundle::default()
    };
    candidate.digest = Some(candidate.canonical_digest_selection().unwrap());
    let producer = candidate.admit().expect("real static producer admission");
    let subject = QualifiedSubject::new(
        &producer,
        SubjectKind::new("type:monitor".as_bytes().to_vec()).unwrap(),
        SubjectIdentity::new("object:monitor".as_bytes().to_vec()).unwrap(),
    );
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id("package:monitor-observations"),
            revision: id("1"),
            digest: Digest::new([1; 32]),
        },
        producer,
        binding: ObservationBinding {
            identity: id("binding:monitor"),
            source_identity: source(),
            schema_identity: id("schema:monitor"),
            signal_identity: id("signal:monitor"),
            trigger_identity: id("trigger:monitor"),
            unit: id("unit:reading"),
            subject_kind: subject.kind().clone(),
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unassigned"),
            membership_rule_identity: id("unassigned"),
            membership_digest: Digest::new([0; 32]),
            membership_document: vec![],
            required_member_identities: vec![id("member:monitor")],
            observation_sources: vec![source()],
            completeness_dependencies: vec![id("completeness:monitor")],
            progress_dependencies: vec![id("progress:monitor")],
            clock_identity: id("clock:monitor-events"),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id("closure:monitor")),
            closure_digest: Some(Digest::new([2; 32])),
            kind: ScopeKind::Window {
                window_identity: id("window:monitor"),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 30,
            },
            members: vec![Member {
                object_identity: id("member:monitor"),
                record_identity: id("unassigned"),
                anchor: Anchor::TimestampNanos(10),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unassigned"),
            binding_identity: id("binding:monitor"),
            source_identity: source(),
            schema_identity: id("schema:monitor"),
            subject,
            signal_identity: id("signal:monitor"),
            trigger_identity: id("trigger:monitor"),
            unit: id("unit:reading"),
            value: ValueState::Present {
                value_type: id("type:reading"),
                canonical_value: "1".to_owned(),
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
            event_time_nanos: 10,
            ingestion_time_nanos: 11,
            causal_relationship_identity: None,
            clock_identity: id("clock:monitor-events"),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 0,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    let record = authority::observation::record_identity(&request.records[0], limits).unwrap();
    request.records[0].identity = record.clone();
    request.scope.members[0].record_identity = record;
    authority::population::assign_request_identities(&mut request, limits).unwrap();
    let qualified = match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("observation owner refused fixture: {other:?}"),
    };
    let subject = SubjectSelection {
        scope_identity: id("window:monitor"),
        population_identity: qualified.scope().population_identity.clone(),
    };
    let owner = AuthoritySelection {
        definition_identity: id("definition:monitor-progress"),
        definition_revision: id("1"),
        definition_digest: Digest::new([3; 32]),
    };
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let base = progress::Selection::new(
        clock(),
        vec![source()],
        TemporalBoundary::TimestampedEvent {
            lower_nanos: 0,
            upper_inclusive_nanos: 29,
            carrier_end_exclusive_nanos: 30,
            watermark_nanos: 30,
        },
        OpenClosed::Closed,
        id("trigger:monitor"),
        authority::observation::CutoffSelection::new(
            id("clock:monitor-events"),
            id("1"),
            40,
            authority::observation::CutoffRule::IngestionTimeAtOrBefore,
            id("1"),
        ),
        id("restoration:monitor"),
    );
    let selection = match rows {
        Some(rows) => base.with_association(rows),
        None => base,
    };
    let document = progress::derive(context, &selection, progress_limits)?;
    let view = progress::read(document.bytes(), context, &selection, progress_limits)?;
    Ok((view, subject))
}

use qsl_analyze::monitor_root::PlanRoot;
use qsl_semantics::check::TimeSourceKind;

fn node(key: quire_exact::NodeKey) -> progress::NodeIdentity {
    progress::NodeIdentity::from_key(key, authority::Limits::owner_max()).unwrap()
}

pub fn row(root: &PlanRoot) -> progress::AssociationRow {
    let source = &root.checked_unit().package().graph().time_sources()[0];
    let claim = root.plan().subject();
    progress::AssociationRow {
        model: source.model.clone(),
        model_node: node(source.model_node.clone()),
        claim: progress::ClaimOccurrence::new(
            node(claim.clause_node().clone()),
            claim.occurrence().origin(),
        ),
        source: match &source.source {
            TimeSourceKind::Dense => progress::TimeSource::Dense,
            TimeSourceKind::Tick { operation, period } => progress::TimeSource::Tick {
                operation: operation.clone(),
                operation_node: node(source.tick_operation_anchor.as_ref().unwrap().clone()),
                period_numerator: period.numerator().to_string(),
                period_denominator: period.denominator().to_string(),
            },
        },
        unit: source.unit.clone(),
        unit_node: node(source.unit_node.clone()),
        time_dimension: node(source.time_dimension.clone()),
        observation_source: self::source().as_str().to_owned(),
        observation_clock: "clock:monitor-events".to_owned(),
        observation_revision: "1".to_owned(),
    }
}
