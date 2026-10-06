// SPDX-License-Identifier: AGPL-3.0-or-later
//! `quire-driver`: the orchestrating driver crate (ADR-011 §6.1 driver row,
//! T-13; ADR-012 §7; FR-001).
//!
//! The driver is the one crate that calls both QSL and CG. It lives
//! downstream of both, because CG depends on QSL (`qsl-replay`) as a normal
//! edge and Cargo refuses a package cycle. No QSL or CG crate depends on it.
//!
//! [`drive`] runs one request through these steps, in this order:
//!
//! 1. S1 to S4: QSL's typed `parse`, `select` and `check` operations.
//! 2. E4: QSL's `package` operation emits the `quire.checked-package/v2`
//!    bytes and their `package_id` ([`Driven::emitted`]).
//! 3. The `route` candidate step: each item's candidate set. The items are
//!    not supplied by the caller: they are written from the compiled unit's
//!    own checked requirement records
//!    (`qsl_package::CheckedPackage::graph().requirements()`, FR-062,
//!    FR-075), one item per record in occurrence-key order
//!    (`qsl_route::request::items_from_requirements`).
//! 4. The pre-negotiation conversion (ADR-012 §7.2, MD-2): every registered
//!    `BackendId` goes through CG's one total conversion
//!    (`BackendKind::from_identity`). When any has no CG kind, the run is
//!    refused with `invalid_capability`/`unknown-backend`, naming each
//!    unconverted id in bytewise order, and nothing is negotiated.
//! 5. E7: CG `negotiate_backend_provider` over the FR-331 envelope built
//!    from the registry and the candidate sets.
//! 6. The `route` routing step, after negotiation, over the items settled
//!    `supported` only. Each routed item carries its `BackendId` and its CG
//!    backend kind ([`Routed`]).
//! 7. CG generation: `generate_routed` over the routed items, each with its
//!    request index, its node, its routed backend and its CG kind, and the
//!    caller's [`GenerationInput`]. The node is the E5 package's
//!    `CheckedNodeId` the item's `WireNodeId` names.
//!
//! E5 runs after step 2: IR's checked-package reader admits the emitted v2
//! bytes, and the admitted `package_id` must be the emitted one
//! ([`Driven::checked`]).
//!
//! Command-level refusals are [`DriverRefusal`]. AGE-394 (the lifecycle and CLI
//! design) renders them.
//!
//! The registry is an ordinary value the caller builds with
//! [`build_registry`] from provider manifests and passes in. It is never
//! global state (ADR-012 §7.1).

#![forbid(unsafe_code)]

pub mod cache;
pub mod cli;
mod drive;
pub mod execute;
mod handoff;
pub mod limits;
pub mod provider;
mod refusal;
mod registry;
mod upstream;

pub use cache::{
    run_cached, Cache, CacheCodecError, CacheEligibility, CacheExclusion, CacheFault, CacheLookup,
    CacheOperation, CacheRequest, CacheRequestError, CacheValueCodec, CacheWriteOutcome,
    CachedRunError,
};
pub use drive::{drive, CompileInput, Driven, GenerationInput, KaniInput, Routed};
pub use execute::{execute, ExecutionEngine, ExecutionFailure, ExecutionRequest, ExecutionResult};
pub use limits::{LimitError, LimitOverrides};
pub use provider::{
    KaniExecutionLimits, KaniProvider, Provider, ProviderItemFailure, ProviderItemResult,
    ProviderLimits, ProviderOutcomeError, ProviderRequest, ProviderResults, ProviderTerminalError,
};
pub use refusal::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};
pub use registry::{
    build_linked_providers, build_registry, AdmittedRegistry, LinkedProviders, ProviderManifest,
};
pub use upstream::{QslCg, Upstream};
