# quire-driver

The `quire-driver` library package orchestrates QSL ADR-011 T-13, with IR
admission between QSL and CG (`quire-contract-codegen`). It remains downstream
of those libraries and does not depend on the concrete `quire-cache` package.
The cache is a separate downstream component that implements the driver's
cache interface. The current `quire` binary target is in the `quire-driver`
package and cannot depend on that component; IR-607 moves the binary to the
separate `quire-cli` crate before cache composition.

`drive` runs one request:

1. S1 to S4 and the E4 v2 emit, through QSL's typed `parse`, `select`, `check` and `package` lifecycle, sharing one `Cancel` handle.
2. E5: IR's `read_checked_package` reads the emitted bytes; the admitted
   package must carry the emitted `package_id`.
3. The `route` candidate step, for each item to negotiate. The items are not
   supplied by the caller: they are written from the compiled unit's own
   checked requirement records
   (`qsl_package::CheckedPackage::graph().requirements()`, QSL-266), one item
   per record in occurrence-key order
   (`qsl_route::request::items_from_requirements`). A unit with no scalar
   operation application writes no requirement record, so the driver routes
   nothing for it and calls neither `route` nor `generate`.
4. The pre-negotiation conversion: every registered `BackendId` through CG's
   `BackendKind::from_identity`. Any id with no CG kind refuses the run with
   `invalid_capability`/`unknown-backend`, naming every unconverted id in
   bytewise order, before any negotiation.
5. E7: CG `negotiate_backend_provider`.
6. The `route` routing step over the items settled `supported` only.
7. CG `generate_routed` over the routed items, each with its request index,
   its admitted `CheckedNodeId`, its routed backend and its kind, and the
   caller's `GenerationInput` (for Kani: subject path and unwind). CG
   derives each routed item's Kani claim from the node's own operation and
   bound (IR-294, IR-298, IR-302):
   a unary operation over a bounded parameter, and a binary operation with
   a bounded operand beside a literal or beside another independently
   bounded operand (QSL's own narrowing `conversion` shape), all settle
   `Supported`. Real limits remain: an operand that is neither bounded nor
   a literal (a plain-`Integer` parameter, a nested subexpression, or a
   negative literal, which QSL emits as a negate node) still settles
   `RequiresBound`; a node narrowed to more than one distinct bound, or
   consumed by both a narrowing conversion and any other node, still
   settles `Unsupported`/`AmbiguousBound` (several narrowing conversions to
   the *same* bound dedupe and settle `Supported`), until IR-304; and path
   conditions are ignored until IR-304 -- CG ranges every operand over its
   full declared domain regardless of the `if` guard that reaches it, live
   or proved dead. The derived claim map is returned in
   `generation.claim_map`, `Some` whenever a Kani group ran. CG also
   returns the FR-014 oracle crate it generated in
   `generation.oracle_artifacts` (IR-300), which the driver does not yet
   write out (Kani execution is blocked on IR-301).

The registry is a value built with `build_registry` from provider manifests
and passed in as an argument. E5's evidence is the caller's domain package
documents and QSL's root feature; no artifact digest evidence is supplied.

## Package boundaries

Keep `quire-cache` outside the `quire-driver` dependency graph. The driver
library defines the cache interface. After IR-607 moves the CLI into a separate
crate, that crate can compose the downstream cache implementation.

The `quire-plugin-host` workspace package owns the process transport, framing,
and lifecycle described by QSL ADR-029 PL-2 through PL-7. Driver dispatch and
typed settlement through CG's process-provider kind remain a separate
FR-006/TC-013 integration step gated on CG's IR-629 API.

## Commands

See `CLAUDE.md`. `make ci` runs every gate.

## License

AGPL-3.0-or-later. See `LICENSE` and `CONTENT_RIGHTS.md`.
