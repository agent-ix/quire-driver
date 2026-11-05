// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retained request to QSL-selected frame replay preparation (FR-009).

use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{DependencyInput, DependencyInputRefusal, SuppliedLibrary};
use quire_contract_codegen::{
    FrameReplay, FrameReplayError, FrameReplayInputs, SelectedFrameTarget,
};
use quire_contract_model::ContractSourceV1;

use crate::{Driven, DriverFault, DriverRefusal};

/// The original owner-specific failure of frame replay preparation.
#[derive(Debug, thiserror::Error)]
pub enum FrameReplayFailure {
    /// QSL refused the supplied dependency selection before call-site selection.
    #[error(transparent)]
    Dependencies(DependencyInputRefusal),
    /// QSL refused call-site selection; the complete typed payload is unchanged.
    #[error(transparent)]
    CallSite(Box<qsl_replay::CallSiteRefusal>),
    /// The selected frame is not associated with the retained driver run.
    #[error(transparent)]
    Driver(DriverRefusal),
    /// CG refused preparation; the complete typed payload is unchanged.
    #[error(transparent)]
    Codegen(FrameReplayError),
}

impl Driven {
    /// Prepares one operation-frame replay using QSL's exact declaring-owner target.
    ///
    /// `inputs.selected_target` must be `None`: the driver supplies that value from
    /// QSL. The selected item must name this run's admitted package and an original
    /// request's complete occurrence. Public request inventory edits do not change
    /// that retained association. The source, domains and dependencies must reproduce
    /// this run's emitted package. No request item, settlement or terminal is created.
    ///
    /// The returned CG preparation exposes its packet/request and may be replayed
    /// through `FrameReplay::replay` under CG's refusal and settlement contracts.
    ///
    /// # Errors
    /// Returns the unchanged QSL or CG failure, or a driver `runtime_invariant`
    /// fault for missing, prepopulated or mismatched selection associations.
    pub fn prepare_frame_replay(
        &self,
        mut inputs: FrameReplayInputs<'_>,
    ) -> Result<FrameReplay, FrameReplayFailure> {
        let dependencies = DependencyInput::new(
            inputs
                .run
                .dependencies
                .iter()
                .map(|dependency| SuppliedLibrary {
                    identity: dependency.identity.clone(),
                    source: SourceIdentity::new(
                        &dependency.source.authority,
                        &dependency.source.identity,
                    ),
                    path: dependency.source.identity.clone(),
                    bytes: dependency.source.bytes.clone(),
                })
                .collect::<Vec<_>>(),
        )
        .map_err(FrameReplayFailure::Dependencies)?;
        let located = qsl_replay::call_site(
            SourceIdentity::new(&inputs.run.source.authority, &inputs.run.source.identity),
            inputs.run.package_name.clone(),
            &inputs.run.source.identity,
            &inputs.run.source.bytes,
            inputs
                .packages
                .iter()
                .map(|document| document.bytes.as_slice()),
            &dependencies,
            &inputs.operation,
        )
        .map_err(FrameReplayFailure::CallSite)?;
        let mismatch = || {
            FrameReplayFailure::Driver(DriverRefusal::Fault(DriverFault::FrameReplayAssociation))
        };
        let selected = &inputs.selected_item;
        let original = self
            .replay_item(selected.request_index)
            .ok_or_else(mismatch)?;
        if inputs.selected_target.is_some()
            || located.package != self.emitted().bytes()
            || selected.package != self.checked()
            || original.index().get() != selected.request_index
            || original.occurrence() != selected.occurrence
            || located.site.frame_occurrence != *selected.occurrence
        {
            return Err(mismatch());
        }
        inputs.selected_target = Some(SelectedFrameTarget {
            operation: located.site.frame_operation,
            package_id: ContractSourceV1 {
                name: self.checked().name().into(),
                lock: self.checked().lock().clone(),
            },
            request_index: selected.request_index,
            occurrence: located.site.frame_occurrence,
        });
        FrameReplay::new(inputs).map_err(FrameReplayFailure::Codegen)
    }
}

/// Stable ownership classification for a frame preparation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameReplayFailureCode {
    /// QSL dependency admission refused.
    Dependencies,
    /// QSL operation selection refused.
    CallSite,
    /// The driver detected a broken retained association.
    RuntimeInvariant,
    /// CG frame preparation refused.
    Codegen,
}

impl FrameReplayFailureCode {
    /// The stable boundary code; native details remain on `FrameReplayFailure`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dependencies => "frame_dependencies",
            Self::CallSite => "frame_call_site",
            Self::RuntimeInvariant => "runtime_invariant",
            Self::Codegen => "frame_codegen",
        }
    }

    /// Every stable frame preparation boundary code.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Dependencies,
            Self::CallSite,
            Self::RuntimeInvariant,
            Self::Codegen,
        ]
    }

    /// Parses a stable boundary code.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

impl FrameReplayFailure {
    /// The stable ownership code, without interpreting any native message.
    pub fn code(&self) -> FrameReplayFailureCode {
        match self {
            Self::Dependencies(_) => FrameReplayFailureCode::Dependencies,
            Self::CallSite(_) => FrameReplayFailureCode::CallSite,
            Self::Driver(_) => FrameReplayFailureCode::RuntimeInvariant,
            Self::Codegen(_) => FrameReplayFailureCode::Codegen,
        }
    }
}
