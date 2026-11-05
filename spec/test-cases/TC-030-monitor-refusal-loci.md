---
id: TC-030
title: "Derive exact monitor target and unsupported-operation loci"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-016
    type: verifies
---
# TC-030: Derive exact monitor target and unsupported-operation loci

## Description

Verify the functional diagnostic constructors of
[FR-016](../functional/FR-016-retain-monitor-refusal-loci.md) against actual
strictly admitted checked nodes and body terms. The test does not accept a
payload merely because an enum can be constructed or displayed.

## Test Procedure

1. Obtain ordinary checked application nodes through the existing public QSL
   compiler and strict IR reader. Obtain a complete valid monitor root with
   a buffer from the public producer/reader path or an existing owner fixture
   consumed by reference. Retain original selectors, structural keys, typed
   terms and actual occurrence lists. Do not copy an upstream fixture corpus,
   fabricate a semantic ID or use a fake preparation entry.
2. Derive the root's uncertainty locus and each permitted target-premise
   location; derive a buffer capacity location from its actual row. Independently
   resolve each resulting path against the unchanged admitted body and assert
   that it ends at the requested binding's existing value, not the Binding
   term itself, and preserves its genuine owner. Require the final child index
   zero for all target premises and each buffer capacity; assert that the
   preceding path resolves to that actual Binding whose value is the endpoint. Verify that the original graph-local selector and resolved content
   key are retained separately.
3. Select actual root applications at empty paths. Invoke `unsupported_at`
   with the kind already decoded from each containing admitted node. Compare
   the returned complete operation with that actual root record, including
   the IEEE conversion's original law, mode and member data. Selecting a
   second actual root operation must retain its different original record.
   Strict IR admits no Application below a node body root; no nested-application
   fixture or conditional observation supplies coverage. For actual nonempty
   child paths, explicitly require admitted Binding, Aggregate and Reference
   terms and a None application record. Check their exact child paths and
   reference stopping boundary. No runtime support-policy verdict is asserted.
4. Exercise an unavailable child, binding child other than zero, continuation
   through a reference and a foreign role/ordinal. Require their exact typed
   InvalidBodyPath or OccurrenceMismatch errors. An ordinary admitted node
   passed as a monitor root yields NotMonitorRoot; an unresolved selector yields
   UnknownNode. Missing/excessive buffer indices and an index on a nonbuffer
   premise yield InvalidPremiseIndex. Do not branch on error messages.
5. Pass None for a genuine generated root, independently assert its actual
   Generated role/ordinal, and require that the diagnostic still retains None
   rather than choosing that available occurrence. Use a real occurrence from an admitted node for
   the positive role/ordinal control; no arbitrary source-map first row is
   selected. Drop the package and path/occurrence input buffers and inspect the
   owned payload's original fields afterward.
6. Compare admitted values and correspondence before/after construction and
   inspect the new module's dependencies/call paths. Require no parser,
   initializer, RT build, copied graph/index, identity recomputation, target
   comparison or runtime-support policy in the diagnostic implementation.

7. Bounded-copy controls: invoke `copy_monitor_locus` directly with
   the original already-resolved node/key/path/occurrence from steps 1–5.
   Compare every canonical field and the entire path, including target
   binding child zero. Also copy an unresolved selector with an independently
   supplied key to establish that copying makes no resolution/validity claim.
   Exercise caller-selected None and actual native role/ordinal. Drop inputs
   and the package, then inspect the returned locus. Use nested canonical
   Node Arc segments with two parents sharing a child; require the original
   Arc backing to be shared and the outer package/Name/path storage to be
   independently owned. Obtain admitted examples from owner paths; direct
   public canonical structs may additionally test copying mechanics but grant
   no admission/monitor-selection evidence.
8. Operating controls: independently calculate the documented work
   schedule and logical retained/scratch storage using native sizes, input
   capacities and each reached edge (count shared descendants per edge).
   Require exact work and peak-storage success, and a one-under stop for each
   limit separately. Include multibyte UTF-8 names, long native paths and
   deep shared keys without a depth cutoff. Pre-cancel with a genuine Cancel
   cause and require the same cause before allocation. Use deterministic
   private reservation/capacity test seams to inject TryReserveError and
   excess text capacity; require original Allocation or Capacity respectively,
   no boxed-text shrink, no returned partial locus, and cleanup of all
   outstanding component-owned reservations. These seams may exercise the
   production reservation adapter; no mocked successful native operation or
   duplicate copier supplies evidence. Exercise checked-arithmetic overflow
   at that adapter with synthetic sizes, without attempting huge allocation.
9. Inspect the production copy path and its reservation adapter for all new
   allocations, Arc retention and cleanup. Require precharge and fallible
   reservation, native canonical fields, no allocating Clone and no hidden
   call to the unbounded resolution constructors. Confirm that private native
   operation/projection-error copying, selection failure assembly, caller-input
   acquisition and full Program preparation still have their named Planned
   owner boundaries. These source inspections establish AC-12 only, not test
   execution credit for opaque upstream allocation.

## Expected Results

Every applicable FR-016 criterion returns the exact derived original locus,
original typed operation data or typed constructor error. A wrong path,
foreign occurrence, reminted identity or operation inferred from a label fails
the test. The four focused `monitor_refusal` tests have passed for the diagnostic
constructor controls of FR-016-AC-1 through AC-7. AC-8 remains a source inspection,
with no Test trace. This case establishes no full FR-015 preparation,
FR-276 construction or temporal-execution acceptance.


The bounded-copy controls have focused implementation evidence for AC-9 through
AC-11: public native-copy tests, an actual admitted target-locus comparison, and
private checked-arithmetic, native reservation-failure and text-capacity controls.
AC-12 has source-inspection evidence only. Successful locus copying does not
qualify existing target/unsupported constructors or selection/projection
failures for bounded diagnostic retention.
