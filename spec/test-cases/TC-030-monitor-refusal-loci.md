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
3. Select an actual application, including an inline application under a
   binding or aggregate when present. Invoke `unsupported_at` with the kind
   already decoded from its containing admitted node. Compare the returned
   complete operation with that actual typed subterm, including its original
   operation-specific data. A second real operation/path must yield the second
   record; a nonapplication terminal yields None. No supported/unsupported
   runtime verdict is asserted by this constructor test.
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

## Expected Results

Every applicable FR-016 criterion returns the exact derived original locus,
original typed operation data or typed constructor error. A wrong path,
foreign occurrence, reminted identity or operation inferred from a label fails
the test. Status is Planned until execution; this case establishes no full
FR-015 preparation, FR-276 construction or temporal-execution acceptance.
