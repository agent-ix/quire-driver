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
---
# FR-019: Prepare the complete monitor program from one plan root under one preparation budget

## Description

When a caller prepares one QSL tick-monitor plan for the runtime, the driver
SHALL take the plan's `PlanRoot`, its independent selection and one finite
caller-owned `NativeJoinBudget`, and SHALL return one runtime-owned
`Rc<monitor::Program>` or one typed first refusal. The driver SHALL pass that
one budget to QSL's row adapters and Contract IR's closure read and SHALL
charge its own work to it, so that every preparation charge precedes the work
or allocation it pays for.
[FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns the admission
and failure-category contract; [FR-017](FR-017-lower-and-retain-admitted-monitor-programs.md)
owns arena lowering. This requirement fixes the request, the budget phases,
the `Layout` derivation, the refusal order and the consumed QSL interface.
The pipeline architecture is
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
| `plan_root` | `&'a qsl_analyze::monitor_root::PlanRoot` | The producer output; the driver reads its emission, adapters and authority handoff only through its read-only accessors. |
| `evidence` | `&'a CheckedPackageEvidence` | Strict-read evidence for the emission. |
| `read_limits` | `CheckedPackageReadLimits` | Strict-read limits. |
| `package` | `&'a CheckedPackageV2` | The independently selected admitted package instance. |
| `selection` | `MonitorSelection<'a>` | The independently captured package and root selector. |
| `package_lock` | `&'a CheckedPackageLockV2` | The independently selected lock. |
| `root_key`, `entry_keys` | `&'a CheckedNodeId`, `[&'a CheckedNodeId; 4]` | Expected root and `init`, `on_event`, `on_watermark`, `on_gap` content keys. |
| `clause`, `subject` | `&'a CheckedNodeId` | Independently selected clause and its `over` parameter. |
| `occurrence` | `&'a CheckedSourceMapEntry` | The selected claim occurrence's authentic source-map row. |
| `profile` | `&'a CheckedSelection` | The selected temporal profile. |
| `counter`, `tick_unit`, `counter_width_bits`, `max_reading_gap`, `max_event_rate`, `uncertainty`, `buffers`, `activation_capacity` | as in FR-015 | Independently selected target premises. |

The request SHALL NOT carry operating limits, caller-built witness rows,
`ProgramParts`, a `TypeEnvironment`, a body, a callback, order or progress
bytes, or a `PlanId`. `MonitorPreparationLimits` and the driver's private
operating ledger SHALL NOT exist; there is no second meter and no adapter
from the old request.

### Consumed QSL interface

The driver SHALL consume exactly these QSL capabilities. Each returns
IR-owned row types or values, takes the caller's budget where it allocates,
and preserves the IR denied charge and locus unchanged on denial:

| Capability | Returns | Required behaviour |
| --- | --- | --- |
| `PlanRoot::emission()` | `&Emission` | Read-only; the same emission as the plan and obligations. |
| `Emission::identity_rows(package, &mut budget)` | `SelectedExactIdentities` | Charges the native join and every owned record, member and union allocation; no partial rows. |
| equality witness adapter `(package, &mut budget)` | IR `SelectedEqualityApplicationRow` and `SelectedEqualityFactRow` slices | Charges each copied row before allocation. |
| source enum, model enum shape and source collection adapters `(&mut budget)` | IR `SelectedSourceEnumRow`, `SelectedModelEnumShapeRow`, `SelectedSourceCollectionRow` (with `key_node`) | QSL performs the conversion to IR rows and charges each owned allocation; the driver performs no field-for-field conversion. |
| Unbudgeted row borrows (`selected_tuple_declaration_rows`, `selected_source_unit_rows`, `selected_model_enum_declaration_rows`, `selected_model_enum_member_rows`, `selected_source_tuple_rows`) | IR row slices | Borrowed; no allocation. |
| Same-emission authority handoff | order value bytes, association value bytes, final clause declaration, claim occurrence, time-source kind and exact tick period | QSL FR-251-AC-45..AC-48; obtained only from this `PlanRoot`; typed refusal when the order premise or association is absent or does not match. |

A QSL refusal SHALL be retained unchanged in `MonitorPreparationError`.

## Outputs

On success the driver SHALL return `Program::try_new(parts)`'s owner. The
caller's budget then holds the cumulative preparation consumption, and the
driver retains no borrow of the request, plan root, package or budget.

On failure the driver SHALL return the first refusal with no program and no
partial parts. `MonitorPreparationErrorCode` SHALL classify: an authority
handoff refusal or `SelectedMonitorTypeClosureErrorCode::Authority` as
`selection`; a denied work or storage charge from any phase as `work` or
`storage`; other QSL row and IR closure refusals as `projection`; RT
`ProgramRefusal` as `program`; cancellation as `cancelled`.

## Behavior

### Phases and charges

The driver SHALL run these phases in order, calling `budget.resume()` at the
start of each and polling `cancel` before each:

| Phase | Work | Charged by, before the step |
| --- | --- | --- |
| P1 Selection | Strict emission read; final-subject admission, partial selection and clock admission of FR-015 | Driver, through `accounted_charge`, for every comparison, inspection and owned allocation that FR-015 charges; native acquisition, package equality and owner declaration lookup stay outside the budget as FR-015 states. |
| P2 Identity rows | `identity_rows` | QSL. |
| P3 Row adapters | equality, source enum, model enum shape, source collection | QSL. |
| P4 Authority row | Read the handoff; `SelectedMonitorAuthorityRow::try_new` with the package's `admission_binding()` | Driver: one handle per byte value and one storage unit per copied byte, locus `AuthorityRow`. |
| P5 Closure | `project_selected_type_closure` with every row and the authority row | Contract IR. |
| P6 Lowering | FR-017 arenas, canonical `TypeEnvironment`, `Layout`, `ProgramParts` | Driver: every inspection as work and every owned container, element, key and copied byte as storage, locus `PreparationOutput(k)`. |

Storage SHALL be counted in IR FR-358's logical units and is never released
or refunded. A denied charge SHALL stop the phase with the budget's first
unpaid kind, amount and locus. The preparation budget SHALL end when the
complete `ProgramParts` is moved into `Program::try_new`; RT construction is
not charged to it, and `build` and every step use the caller's `StepBudget`.
Plan capacities are never budget inputs.

### Layout derivation

| `Layout` field | Value |
| --- | --- |
| `root` | `layout_context().root_key()` as a canonical Exact `NodeKey`. |
| `clause` | `layout_context().clause_key()` as a canonical Exact `NodeKey`. |
| `package_lock` | `layout_context().package_lock()` bytes. |
| `subject` | `layout_context().subject()`. |
| `occurrence` | `layout_context().occurrence()` bytes. |
| `profile` | `layout_context().profile()`. |
| `clock_binding` | `layout_context().clock_binding()` bytes. |
| `admitted_order`, `progress_context` | The receipt's `order_value()` and `progress_association()` bytes. |
| `state`, `inputs`, `scope`, `results`, `step_result` | The plan's admitted type nodes with their canonical `ValueType` from the closure. |
| `entries` | Arena addresses of the four lowered entries. |
| `counter`, `tick_unit`, `time_dimension`, `counter_width_bits`, `max_reading_gap`, `max_event_rate`, `uncertainty`, `buffers`, `activation_capacity` | The admitted plan premises, after the FR-015 comparison with the independent target, with exact values. |

Each byte and text value SHALL be copied exactly once, under P6. No field
SHALL be defaulted, re-encoded, hashed or taken from QSL's `PlanId`.

### Refusal order

The first failing phase SHALL determine the refusal. Within a phase, the
owning component's order applies: FR-015 for P1, QSL for P2 and P3, `try_new`
for P4, IR FR-357 and FR-361 for P5, FR-017 for P6, and RT for construction.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-019-AC-1 | From a genuine QSL plan root for the 20-bit 1 MHz deadline claim, `prepare_monitor` returns a Program whose `Layout` fields equal, byte for byte, the IR layout context, the receipt bytes, QSL's order value and the observation owner's association value, with the exact admitted premises and four entry addresses; the program builds after the plan root, package and request are dropped. | Test |
| FR-019-AC-2 | One budget measured over a successful preparation succeeds at its exact work and storage limits. For each of P1 through P6, a limit one unit below that phase's first charge returns `work` or `storage` with the budget's first unpaid kind, amount and phase locus, no program and no reset of earlier consumption. | Test |
| FR-019-AC-3 | A QSL adapter or `identity_rows` denial and a QSL authority handoff refusal (absent order premise, absent association, second emission) are returned unchanged inside `MonitorPreparationError`, classified `work`, `storage` or `selection`. | Test |
| FR-019-AC-4 | Each IR authority refusal code reaches the caller unchanged, classified `selection`; a handoff from a second emission of the same source returns `ForeignEmission`. | Test |
| FR-019-AC-5 | With defects placed in two phases at once, preparation returns the earlier phase's refusal; cancellation set before each phase returns `cancelled`, no program and the consumption admitted so far. | Test |
| FR-019-AC-6 | A reached map collection lowers into the canonical `TypeEnvironment` with its key and value types; changing only QSL's `PlanId` metadata leaves `Layout` unchanged, while changing an entry body changes `Layout.root`. | Test |
| FR-019-AC-7 | The budget's consumption is unchanged by `Program::try_new`; an RT construction refusal is returned as `program` with its original `ProgramRefusal`. | Test |
| FR-019-AC-8 | Source and API inspection finds one `&mut NativeJoinBudget` parameter on `prepare_monitor`, `lower_scalar_functions` and `copy_monitor_locus`; no `MonitorPreparationLimits`, private ledger, storage release, driver-side QSL-to-IR row conversion, `PlanId` read, digest, compatibility path or second meter. | Inspection |

## Dependencies

- QSL FR-251 owns the plan root, its adapters and the authority handoff.
- IR FR-357, FR-358 and FR-361 own the closure, the budget and the receipt.
- RT FR-276 and interface-002 own `ProgramParts` and `Program::try_new`.
- [TC-033](../test-cases/TC-033-prepare-monitor-under-one-budget.md) defines
  the evidence; the genuine producer criteria run in `quire-integration`.
