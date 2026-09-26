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
   caller's `GenerationInput` (for Kani: claim map, subject path, unwind,
   attestation; pins are `KaniToolPins::pinned()`).

The registry is a value built with `build_registry` from provider manifests
and passed in as an argument.

## Not wired yet

- **Claim map.** The caller supplies the Kani claim map; IR-294 has CG
  derive it.
- **Requirement records.** `qsl-package` does not expose a `CheckedPackage`'s
  per-item requirement records, so the caller passes the requested items.

## Commands

See `CLAUDE.md`. `make ci` runs every gate.

## License

AGPL-3.0-or-later. See `LICENSE` and `CONTENT_RIGHTS.md`.
