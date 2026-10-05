// SPDX-License-Identifier: AGPL-3.0-or-later
//! Linked provider interface and the built-in Kani execution adapter.

use std::{path::Path, time::Duration};

use qsl_package::EmittedPackage;
use qsl_replay::{RequestIndex, TerminalRecord};
use quire_contract_codegen::{
    execute_kani_obligation, run_terminal_value, Candidate, KaniExecutableHarness,
    KaniExecutionEvidence, KaniExecutionRefusal, KaniExecutionRequest, KaniInstallation,
    KaniRunOutcome, KindOutput, RoutedItemOutput, TerminalPairError,
};
use quire_exact::{Cancel, CancelCause};

use crate::ProviderManifest;

/// The generated, routed items selected for one provider invocation.
///
/// A request is borrowed from CG's generation result: item indices, backend
/// identities, records and executable harnesses retain their upstream types.
#[derive(Clone, Copy, Debug)]
pub struct ProviderRequest<'a> {
    items: &'a [RoutedItemOutput],
}

impl<'a> ProviderRequest<'a> {
    /// The items routed to this provider, in request order.
    #[must_use]
    pub fn items(self) -> &'a [RoutedItemOutput] {
        self.items
    }

    /// A borrowed request over exactly `items`.
    #[must_use]
    pub const fn new(items: &'a [RoutedItemOutput]) -> Self {
        Self { items }
    }
}

/// Kani's bounded execution inputs. The caller owns the generated crate and
/// target directories; CG verifies each harness is present in that crate.
#[derive(Clone, Copy, Debug)]
pub struct KaniExecutionLimits<'a> {
    /// The explicitly selected Kani installation.
    pub installation: &'a KaniInstallation,
    /// Directory containing the generated harness source.
    pub crate_directory: &'a Path,
    /// Cargo target directory for Kani's build.
    pub target_directory: &'a Path,
    /// Per-harness wall-clock bound.
    pub timeout: Duration,
}

/// Resource and execution configuration supplied to a provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProviderLimits<'a> {
    /// Kani execution configuration; absent when this provider has no Kani
    /// harnesses in the current binary.
    pub kani: Option<KaniExecutionLimits<'a>>,
}

/// A typed result for one routed Kani item.
#[derive(Debug)]
pub struct ProviderItemResult {
    /// The original request index.
    pub request_index: usize,
    /// The routed backend identity.
    pub backend: Candidate,
    /// Kani's typed evidence or the typed reason execution did not complete.
    pub result: Result<KaniExecutionEvidence, ProviderItemFailure>,
}

impl ProviderItemResult {
    /// Compose the terminal record for a non-falsified Kani result through
    /// CG's authoritative map. A falsification needs QSL replay first.
    pub fn terminal_record(&self) -> Result<TerminalRecord, ProviderTerminalError> {
        let evidence = self
            .result
            .as_ref()
            .map_err(|_| ProviderTerminalError::ExecutionDidNotProduceEvidence)?;
        if matches!(&evidence.outcome, KaniRunOutcome::Falsified { .. }) {
            return Err(ProviderTerminalError::ReplayUnavailableForScalarObligation);
        }
        let value = run_terminal_value(&evidence.outcome, evidence.success_checks, None)
            .map_err(ProviderTerminalError::TerminalPair)?;
        Ok(TerminalRecord::new(
            RequestIndex::new(self.request_index),
            value,
        ))
    }
}

/// Why a Kani item does not yet have a terminal record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderTerminalError {
    /// Kani execution returned a typed refusal instead of evidence.
    ExecutionDidNotProduceEvidence,
    /// The scalar harness requires a replay adapter that is not present in
    /// the locked CG API; no settlement is guessed.
    ReplayUnavailableForScalarObligation,
    /// CG rejected the outcome/settlement pair.
    TerminalPair(TerminalPairError),
}

/// Why one routed item did not yield Kani evidence.
#[derive(Debug)]
pub enum ProviderItemFailure {
    /// Cancellation was already requested before this item's execution began.
    Cancelled(Option<CancelCause>),
    /// The item was not a Kani output or did not contain an executable harness.
    NoExecutableHarness,
    /// Routing supplied an item for another provider identity.
    UnexpectedBackend {
        /// The identity this provider registered.
        expected: String,
        /// The identity the routed item names.
        actual: String,
    },
    /// No Kani execution configuration was supplied.
    MissingExecutionLimits,
    /// CG's execution API refused to run or interpret the backend result.
    Execution(KaniExecutionRefusal),
}

/// Results returned by one provider invocation, in the request's item order.
#[derive(Debug, Default)]
pub struct ProviderResults {
    /// One typed result for every item in the provider request.
    pub items: Vec<ProviderItemResult>,
}

/// A provider linked into the driver binary.
pub trait Provider {
    /// The FR-331 manifest this provider advertises.
    fn manifest(&self) -> ProviderManifest;

    /// Run the routed items with the exact emitted package, limits and
    /// caller-owned cancellation handle supplied by the driver.
    fn run(
        &self,
        package: &EmittedPackage,
        request: &ProviderRequest<'_>,
        limits: &ProviderLimits<'_>,
        cancel: &Cancel,
    ) -> ProviderResults;
}

/// The linked Kani provider, using CG's executor without reclassifying its
/// evidence or execution refusals.
#[derive(Clone, Debug)]
pub struct KaniProvider {
    manifest: ProviderManifest,
}

impl KaniProvider {
    /// Build the provider from the manifest it registers with the QSL registry.
    #[must_use]
    pub const fn new(manifest: ProviderManifest) -> Self {
        Self { manifest }
    }
}

impl Provider for KaniProvider {
    fn manifest(&self) -> ProviderManifest {
        self.manifest.clone()
    }

    fn run(
        &self,
        _package: &EmittedPackage,
        request: &ProviderRequest<'_>,
        limits: &ProviderLimits<'_>,
        cancel: &Cancel,
    ) -> ProviderResults {
        let mut items = Vec::with_capacity(request.items.len());
        for item in request.items {
            let result = if item.backend.identity != self.manifest.identity {
                Err(ProviderItemFailure::UnexpectedBackend {
                    expected: self.manifest.identity.clone(),
                    actual: item.backend.identity.clone(),
                })
            } else if cancel.poll() {
                Err(ProviderItemFailure::Cancelled(cancel.cause()))
            } else {
                run_kani_item(item, limits.kani)
            };
            items.push(ProviderItemResult {
                request_index: item.request_index,
                backend: item.backend.clone(),
                result,
            });
        }
        ProviderResults { items }
    }
}

fn run_kani_item(
    item: &RoutedItemOutput,
    limits: Option<KaniExecutionLimits<'_>>,
) -> Result<KaniExecutionEvidence, ProviderItemFailure> {
    let KindOutput::Kani {
        harness: Some(harness),
        ..
    } = &item.output
    else {
        return Err(ProviderItemFailure::NoExecutableHarness);
    };
    let limits = limits.ok_or(ProviderItemFailure::MissingExecutionLimits)?;
    execute_kani_obligation(&KaniExecutionRequest {
        installation: limits.installation,
        harness: KaniExecutableHarness::from(harness),
        crate_directory: limits.crate_directory,
        target_directory: limits.target_directory,
        timeout: limits.timeout,
    })
    .map_err(ProviderItemFailure::Execution)
}

#[cfg(test)]
mod tests {
    use qsl_replay::TerminalValue;
    use quire_contract_codegen::{Candidate, KaniRunOutcome, KaniSolver};

    use super::{ProviderItemFailure, ProviderItemResult, ProviderTerminalError};
    use quire_contract_codegen::KaniExecutionEvidence;

    fn result(outcome: KaniRunOutcome) -> ProviderItemResult {
        ProviderItemResult {
            request_index: 4,
            backend: Candidate {
                identity: "agent-ix/provider/kani".to_owned(),
            },
            result: Ok(KaniExecutionEvidence {
                kind: None,
                harness_path: "generated::claim".to_owned(),
                launcher_path: "/tools/cargo-kani".to_owned(),
                arguments: Vec::new(),
                unwind: 1,
                solver: KaniSolver::Cadical,
                exit_code: Some(0),
                outcome,
                success_checks: 3,
                checks: Vec::new(),
                batch: None,
            }),
        }
    }

    /// Trace: FR-009-AC-4.
    #[test]
    fn proved_kani_result_uses_cgs_terminal_mapping() {
        let terminal = result(KaniRunOutcome::Verified)
            .terminal_record()
            .expect("verified evidence maps without replay");
        assert_eq!(terminal.request_index().get(), 4);
        assert_eq!(
            terminal.value(),
            &TerminalValue::Proved { success_checks: 3 }
        );
    }

    /// Trace: FR-009-AC-4.
    #[test]
    fn falsified_scalar_does_not_guess_a_replay_settlement() {
        let error = result(KaniRunOutcome::Falsified {
            counterexample: "witness".to_owned(),
        })
        .terminal_record()
        .expect_err("scalar replay API is not in the locked CG interface");
        assert_eq!(
            error,
            ProviderTerminalError::ReplayUnavailableForScalarObligation
        );
    }

    /// Trace: FR-009-AC-4.
    #[test]
    fn execution_refusal_is_not_rewritten_as_a_terminal_outcome() {
        let item = ProviderItemResult {
            request_index: 0,
            backend: Candidate {
                identity: "agent-ix/provider/kani".to_owned(),
            },
            result: Err(ProviderItemFailure::NoExecutableHarness),
        };
        assert_eq!(
            item.terminal_record().expect_err("there is no evidence"),
            ProviderTerminalError::ExecutionDidNotProduceEvidence
        );
    }
}
