---
id: FR-019
title: "Prepare the complete monitor program from one plan root under one preparation budget"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-016
    type: references
  - target: ix://agent-ix/quire-driver/FR-017
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-361
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-010
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-251
    type: depends_on
  - target: ix://agent-ix/quire-contract-runtime/FR-276
    type: depends_on
  - target: ix://agent-ix/quire-contract-runtime/interface_002
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-004
    type: depends_on
---
# FR-019: Prepare the complete monitor program from one plan root under one preparation budget

## Description

When a caller prepares one QSL tick-monitor plan for the runtime, the driver
SHALL take ownership of the plan's `PlanRoot` with its authority selection,
progress view and independent selection, borrow one finite caller-owned
`NativeJoinBudget`, and return one runtime-owned `Rc<monitor::Program>` or one
typed first refusal.

The driver SHALL pass that one budget to QSL's authority admission and row
adapters and to Contract IR's closure read, and SHALL charge its own work to
it, so that every preparation charge precedes the work or allocation it pays
for.

[FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns admission and the
failure categories; [FR-017](FR-017-lower-and-retain-admitted-monitor-programs.md)
owns arena lowering. This requirement fixes the request, the budget phases,
the closure-row mapping, the `Layout` derivation, the refusal order and the
consumed QSL interface. The pipeline architecture is
[IR AD-010](ix://agent-ix/quire-contract-ir/AD-010).

## Inputs

```rust
pub fn prepare_monitor(
    request: MonitorPreparationRequest<'_>,
    budget: &mut quire_contract_model::NativeJoinBudget,
    cancel: &quire_exact::Cancel,
) -> Result<std::rc::Rc<quire_contract_runtime::monitor::Program>, MonitorPreparationError>;
```

`MonitorPreparationRequest` SHALL have exactly these public fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `plan_root` | `qsl_analyze::monitor_root::PlanRoot` | The producer output, owned; consumed by authority admission. |
| `authority_selection` | `qsl_analyze::monitor_root::PlanAuthoritySelection<'a>` | The independent source, claim, observation subject, clock and source selection. |
| `progress` | `Option<quire_observation::authority::progress::View>` | The observation owner's admitted progress view (quire-observation FR-004-AC-7). |
| `authority_caps` | `qsl_analyze::monitor_root::PlanAuthorityValidationCaps` | QSL's validation caps for `admit_authority`. |
| `package_lock` | `&'a CheckedPackageLockV2` | The independently selected lock. |
| `root_key`, `entry_keys` | `&'a CheckedNodeId`, `[&'a CheckedNodeId; 4]` | Expected root and `init`, `on_event`, `on_watermark`, `on_gap` content keys. |
| `clause`, `subject` | `&'a CheckedNodeId` | The independently selected clause and its `over` parameter. |
| `occurrence` | `&'a CheckedSourceMapEntry` | The selected claim occurrence's authentic source-map row. |
| `profile` | `&'a CheckedSelection` | The selected temporal profile. |
| `counter`, `tick_unit`, `counter_width_bits`, `max_reading_gap`, `max_event_rate`, `uncertainty`, `buffers`, `activation_capacity` | as in FR-015 | Independently selected target premises. |
| `time_dimension` | `&'a CheckedNodeId` | The independently selected base Time dimension. |

The request SHALL NOT carry operating limits, strict-read evidence or limits,
a package, caller-built witness rows, `ProgramParts`, a `TypeEnvironment`, a
body, a callback, order or progress bytes, or a `PlanId`.
`MonitorPreparationLimits` and the driver's private operating ledger SHALL NOT
exist; there is no second meter and no adapter from the old request.

### Consumed QSL interface (cross-owner seam, QSL-owned)

The driver SHALL consume exactly these QSL capabilities:

| Capability | Returns | Required behaviour |
| --- | --- | --- |
| `PlanRoot::admit_authority`, whose signature QSL owns. By role, it consumes the `PlanRoot` and takes the `PlanAuthoritySelection`, the optional `quire_observation::authority::progress::View`, QSL's `PlanAuthorityValidationCaps`, the caller's `&mut NativeJoinBudget` and the caller's `NativeJoinLocus` | `AdmittedPlanAuthority` or `PlanAuthorityRefusal` | QSL's single strict read of the root's own final emission; every selection, read and canonical-value allocation charged to the budget first; a budget denial reported with the budget's first unpaid charge. |
| `AdmittedPlanAuthority::{root, admitted_package, admission_binding, admitted_order, progress_context}` | the owned root, the admitted package, its `AdmissionBinding`, borrowed order and association values | Read-only; the time source kind and exact period are readable as IR FR-361 requires. |
| `PlanRoot::emission()` | `&Emission` | Read-only; the same emission as the plan and obligations. |
| `Emission::identity_rows(package, &mut budget)` | `ReadIdentityRows` (declarations, fields, source records, source unions) or `IdentityFileRefusal` | Charges the native join and every owned record, member and union clone before it is made. |
| `Emission::equality_witness(package, &mut budget)` | `ReadEqualityRows` or `NativeRowError` | Charges each copied row first. |
| `Emission::selected_source_enum_rows(&mut budget)`, `selected_ir_model_enum_shape_rows(&mut budget)` | IR `SelectedSourceEnumRow`, `SelectedModelEnumShapeRow` vectors or `NativeRowError` | Charge each owned allocation first, with locus `NativeJoinLocusSource::AdapterRow(i)`. |
| `SelectedSourceCollectionRow::to_ir(&mut budget, locus)` | IR `SelectedSourceCollectionRow` or `NativeRowError` | Converts a map row with its `key_node` and `ValueType::Map` under IR FR-357-AC-16; the driver passes locus `AdapterRow(i)` for row `i`. |
| `Emission::{selected_tuple_declaration_rows, selected_source_unit_rows, selected_model_enum_declaration_rows, selected_model_enum_member_rows, selected_source_tuple_rows, selected_source_collection_rows}` | borrowed row slices | No allocation. |

A QSL refusal SHALL be retained unchanged in `MonitorPreparationError`. The
read-only `PlanRoot` accessors, `admit_authority` and `AdmittedPlanAuthority`
are QSL FR-251's handoff (AC-45 through AC-48). The budget parameter,
`admission_binding()`, map conversion in `to_ir`, `AdapterRow` loci and
charged `identity_rows` clones are QSL seam requirements that this
requirement depends on.

## Outputs

On success the driver SHALL return `Program::try_new(parts)`'s owner. The
caller's budget then holds the cumulative preparation consumption, and the
driver retains no borrow of the request, authority, package or budget.

On failure the driver SHALL return the first refusal with no program and no
partial parts. `MonitorPreparationErrorCode` SHALL classify every cause:

| Cause | Code |
| --- | --- |
| A denied work or storage charge in any phase, including inside QSL, IR or FR-016 | `work` or `storage` |
| `PlanAuthorityRefusal` other than a budget denial | `selection` |
| FR-015 selection, target and clock refusals | their FR-015 codes (`selection`, `target_mismatch`) |
| `IdentityFileRefusal` or `NativeRowError` other than a budget denial | `projection` |
| `SelectedMonitorAuthorityRowError` | `selection` |
| `SelectedMonitorTypeClosureErrorCode::Authority(_)` | `selection` |
| Other closure refusals | `projection` |
| A reached operation without an RT form | `unsupported` |
| A key, rational or integer conversion refusal in P7 | `projection` |
| An admitted tick unit whose dimension differs from `time_dimension` | `target_mismatch` |
| A broken driver lowering invariant, including disagreeing entry result types | `internal` |
| RT `ProgramRefusal` | `program` |
| Cancellation | `cancelled` |

## Behavior

### Phases and charges

The driver SHALL run these phases in order, calling `budget.resume()` at the
start of each and polling `cancel` before each:

| Phase | Work | Charged by, before the step |
| --- | --- | --- |
| P1 Authority admission | `request.plan_root.admit_authority`, given by role `request.authority_selection`, `request.progress`, `request.authority_caps`, the caller's budget and the locus `NativeJoinLocus { source: PreparationSelection(0), form: "plan_authority" }`, in QSL's argument order | QSL. |
| P2 Selection | FR-015 final-subject admission and clock admission over `authority.admitted_package()` | Driver, through `accounted_charge` with `PreparationSelection(k)`; the owner's linear declaration lookup is outside the budget. |
| P3 Identity rows | `identity_rows` | QSL. |
| P4 Row adapters | `equality_witness`, `selected_source_enum_rows`, `selected_ir_model_enum_shape_rows`, `to_ir` per collection row | QSL. |
| P5 Authority row | `SelectedMonitorAuthorityRow::try_new` from the authority | Driver, locus `AuthorityRow`: work for each component inspected, and one handle storage unit for the cloned `admission_binding()` handle before cloning it; the row borrows the order and association bytes, so no storage is charged for them. |
| P6 Closure | `project_selected_type_closure` on the plan of `authority.admitted_package()` whose node ID is the selected root | Contract IR. |
| P7 Lowering | `lower_monitor`, canonical `TypeEnvironment`, `Layout`, `ProgramParts` | Driver: every inspection as work and every owned container, element, key and copied byte as storage, locus `PreparationOutput(k)`. |

`authority.admitted_package()` SHALL be the only admitted package: it
supplies the plan ref, the `admission_binding()` used in P5, every adapter's
`package` argument and every `Layout` value. The driver SHALL make no strict
read of its own.

Storage SHALL be counted in IR FR-358's logical units and is never released
or refunded. A denied charge SHALL stop the phase with the budget's first
unpaid kind, amount and locus. The preparation budget SHALL end when the
complete `ProgramParts` is moved into `Program::try_new`; RT construction is
not charged to it, and `build` and every step use the caller's `StepBudget`.
Plan capacities are never budget inputs.

### Closure rows

| `SelectedMonitorClosureRows` field | Source |
| --- | --- |
| `declarations`, `fields`, `source_records`, `source_unions` | `ReadIdentityRows` from P3 |
| `model_tuples` | `emission.selected_tuple_declaration_rows()` |
| `source_units` | `emission.selected_source_unit_rows()`; the copy inside `ReadEqualityRows` is not used for this field |
| `model_enum_declarations`, `model_enum_members` | `emission.selected_model_enum_declaration_rows()`, `selected_model_enum_member_rows()` |
| `model_enum_shapes` | `selected_ir_model_enum_shape_rows` from P4 |
| `source_tuples` | `emission.selected_source_tuple_rows()` |
| `source_enums` | `selected_source_enum_rows` from P4 |
| `source_collections` | the `to_ir` results from P4, in row order |
| `equality_applications`, `equality_facts` | `ReadEqualityRows` from P4 |
| `authority` | the P5 row |

Here `emission` is `authority.root().emission()`.

### Layout derivation

Each value SHALL be copied into `Layout` once, during P7. The key, rational and
integer conversions below decode admitted canonical values into the runtime's
native types; no value is re-encoded. Each conversion charges its work and
output storage first.

| `Layout` field | Source | Conversion |
| --- | --- | --- |
| `root`, `clause` | `layout_context().root_key()`, `clause_key()` | `CheckedNodeId` to `quire_exact::NodeKey` through the owner key codec (`NodeKey::try_new_with`), as FR-017 lowers keys. |
| `package_lock`, `occurrence`, `clock_binding` | `layout_context()` bytes | Copied into `Box<[u8]>`. |
| `subject`, `profile` | `layout_context()` text | Copied into `String`. |
| `admitted_order`, `progress_context` | the receipt's `order_value()`, `progress_association()` | Copied into `Box<[u8]>`. |
| `state`, `inputs`, `scope`, `results` | `plan.state()`, `inputs()`, `scope()`, `results()` and the closure's Exact type for each node | `TypeBinding { key, value_type }`, key converted as above. |
| `step_result` | `plan.on_event().result_type()` and its Exact type | The driver SHALL require `on_watermark().result_type()` and `on_gap().result_type()` to have the same resolved structural key; a difference is `internal`, since strict admission fixes one tuple type. |
| `entries` | the four lowered entry addresses | none |
| `counter`, `counter_width_bits`, `max_reading_gap` | the plan's admitted premises after FR-015's comparison | Copied. |
| `tick_unit` | `plan.tick_unit()` | Key converted as above. |
| `time_dimension` | `SelectedMonitorTypeClosure::tick_unit_dimension()`, the admitted dimension of the plan's `tick_unit` premise that IR's closure seeds | The driver SHALL require structural equality with `request.time_dimension`; inequality is `target_mismatch`. Key converted as above. |
| `max_event_rate`, `uncertainty` | `&CheckedRational` | Canonical numerator and denominator decimal text to `quire_exact::Rational`. |
| `buffers` | each `CheckedMonitorBufferRef` | `Buffer { owner: subformula key converted, capacity: canonical decimal to quire_exact::Integer }`, in admitted order. |
| `activation_capacity` | canonical decimal text | To `quire_exact::Integer`, including zero. |

No field SHALL be defaulted, hashed or taken from QSL's `PlanId`.

### Refusal order

The first failing phase SHALL determine the refusal. Within a phase, the
owning component's order applies: QSL for P1, P3 and P4, FR-015 for P2,
`try_new` for P5, IR FR-357 and FR-361 for P6, FR-017 and the Layout table
order above for P7, and RT for construction.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-019-AC-1 | From a genuine QSL plan root for the 20-bit 1 MHz deadline claim with a genuine progress view, `prepare_monitor` returns a Program whose `Layout` fields equal the IR layout context, the receipt bytes, QSL's order value, the observation owner's association value and the converted admitted premises, with the four entry addresses; no driver strict read occurs, and the program builds after the authority and request are dropped. | Test |
| FR-019-AC-2 | Let `C(p, r)` be the cumulative consumption of resource `r` before phase `p` plus `p`'s first charge of `r`. For every phase and every resource it charges, a budget with `r` limited to `C(p, r) - 1` and the other resource at its measured total refuses in that phase with code `work` or `storage` and the budget's first unpaid kind, amount and phase locus, no program and earlier consumption retained; the measured totals succeed exactly. | Test |
| FR-019-AC-3 | A QSL `admit_authority`, `identity_rows` or adapter budget denial, and a non-budget `PlanAuthorityRefusal` (missing progress, claim, selection, final emission), are returned unchanged inside `MonitorPreparationError` with the codes of the classification table. | Test |
| FR-019-AC-4 | Each IR authority refusal code reaches the caller unchanged as `selection`; a row built from one authority but read against the package of a second admission of the same root bytes returns `ForeignEmission`; a `SelectedMonitorAuthorityRowError` returns `selection`. | Test |
| FR-019-AC-5 | With defects in two phases at once, preparation returns the earlier phase's refusal; cancellation set before each phase returns `cancelled`, no program and the consumption admitted so far. | Test |
| FR-019-AC-6 | A reached map collection lowers into the canonical `TypeEnvironment` with its key and value types; changing only QSL's `PlanId` metadata leaves `Layout` unchanged, while changing an entry body changes `Layout.root`. | Test |
| FR-019-AC-7 | The budget's consumption is unchanged by `Program::try_new`; an RT construction refusal is returned as `program` with its original `ProgramRefusal`. | Test |
| FR-019-AC-8 | Source and API inspection finds one `&mut NativeJoinBudget` parameter on `prepare_monitor`, `admit_monitor_final_subject`, `lower_monitor`, `lower_scalar_functions` and `copy_monitor_locus`; no `MonitorPreparationLimits`, private ledger, storage refund, driver strict read, driver-side QSL-to-IR row conversion, `PlanId` read, digest, compatibility path or second meter. | Inspection |
| FR-019-AC-9 | Each Layout conversion succeeds on genuine admitted values, with `Layout.time_dimension` equal to `tick_unit_dimension()`; a request time dimension unequal to it returns `target_mismatch`; an unconvertible key returns `projection` with the native `InvalidNodeKey`. | Test |
| FR-019-AC-10 | Source inspection shows that disagreeing `on_event`, `on_watermark` and `on_gap` result types map to `internal`, with no fault-injection seam or public closure constructor added to produce them. | Inspection |

## Dependencies

- QSL FR-251 owns the plan root, `admit_authority`, `AdmittedPlanAuthority`
  and the adapters. Its read-only `PlanRoot` accessors and `admit_authority`
  are delivered by QSL's open plan-authority change; this requirement depends
  on that change and on the seam requirements listed above.
- The driver takes a normal dependency on `qsl-analyze` for `PlanRoot` and
  `AdmittedPlanAuthority`.
- IR FR-357, FR-358 and FR-361 own the closure, the budget, the receipt and
  the closure's `tick_unit_dimension()` read.
- quire-observation [FR-004](ix://agent-ix/quire-observation/FR-004) (AC-7)
  issues the progress view and its canonical association value.
- RT FR-276 and interface-002 own `ProgramParts` and `Program::try_new`.
- [TC-033](../test-cases/TC-033-prepare-monitor-under-one-budget.md) defines
  the evidence; the genuine producer criteria run in `quire-integration`.
