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
//! 1. S1 to S4: `qsl_replay::spine::compile`.
//! 2. E4: the same call emits the `quire.checked-package/v2` bytes and their
//!    `package_id` ([`Driven::emitted`]).
//! 3. The `route` candidate step: each requested item's candidate set.
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
//! 7. CG generation with those `BackendId`s. **Not wired.** CG has no
//!    generation entry point keyed by a routed backend; [`Driven::routed`]
//!    is the input it would take. See FR-001 Status. The E5 hand-off to IR
//!    is not wired either.
//!
//! Command-level refusals are [`DriverRefusal`]. AGE-394 (the lifecycle and CLI
//! design) renders them.
//!
//! The registry is an ordinary value the caller builds with
//! [`build_registry`] from provider manifests and passes in. It is never
//! global state (ADR-012 §7.1).

#![forbid(unsafe_code)]

mod drive;
mod refusal;
mod registry;
mod upstream;

pub use drive::{drive, CompileInput, Driven, RequestedItem, Routed};
pub use refusal::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};
pub use registry::{build_registry, ProviderManifest};
pub use upstream::{QslCg, Upstream};
