# quire-driver

The orchestrating driver crate of QSL ADR-011 T-13 (§6.1 driver row). It is
the one crate that calls both QSL and CG (`quire-contract-codegen`), and
IR's reader between them, so it
lives downstream of both. Nothing depends on it.

`drive` runs one request:

1. S1 to S4 and the E4 v2 emit, through `qsl_replay::spine::compile`.
2. E5: IR's `read_checked_package` reads the emitted bytes; the admitted
   package must carry the emitted `package_id`.
3. The pre-negotiation conversion: every registered `BackendId` through CG's
   `BackendKind::from_identity`. Any id with no CG kind refuses the run with
   `invalid_capability`/`unknown-backend`, naming every unconverted id in
   bytewise order, before any negotiation.
4. The `route` candidate step, then E7: CG `negotiate_backend_provider`.
5. The `route` routing step over the items settled `supported` only.
6. CG `generate_routed` over the routed items, each with its request index,
   its admitted `CheckedNodeId`, its routed backend and its kind, and the
   caller's `GenerationInput` (for Kani: subject path, unwind, attestation;
   pins are `KaniToolPins::pinned()`). CG derives each routed item's Kani
   claim from the node's own operation and bound (IR-294, IR-298, IR-302):
   a unary operation over a bounded parameter, and a binary operation with
   a bounded operand beside a literal or beside another independently
   bounded operand (QSL's own narrowing `conversion` shape), all settle
   `Supported`. A plain-`Integer` parameter beside a bounded operand still
   settles `RequiresBound`, and a node with more than one narrowing caller
   still settles `Unsupported`/`AmbiguousBound`, until IR-304. The derived
   claim map is returned in `generation.claim_map`, `Some` whenever a Kani
   group ran. CG also returns the FR-014 oracle crate it generated in
   `generation.oracle_artifacts` (IR-300), which the driver does not yet
   write out (Kani execution is blocked on IR-301).

The registry is a value built with `build_registry` from provider manifests
and passed in as an argument. The diagnostics catalog reference E5's
evidence needs comes from `qsl_package::diagnostics_catalog()` (QSL-260).

## Not wired yet

- **Requirement records.** `qsl_package::CheckedPackage::graph().requirements()`
  exists (QSL-258), but it is empty for every unit this crate compiles today
  (the only shipped family requests no capability kind for a function
  declaration) and, even populated, is keyed by checked declaration, not by
  the arbitrary body node a Kani item names. So the caller still passes the
  requested items directly.

## Commands

See `CLAUDE.md`. `make ci` runs every gate.

## License

AGPL-3.0-or-later. See `LICENSE` and `CONTENT_RIGHTS.md`.
