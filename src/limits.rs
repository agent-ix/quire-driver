// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resolve and apply FR-003's caller-selected QSL limit settings through QSL.

use qsl_foundation::{Setting, SettingLimits, SettingName, UsageRefusal};
use qsl_replay::{
    spine::{Call, ClauseRunRequest, SpineLimits},
    AccountingLimits, CallerLimits, ObservationLimits,
};

/// A recognized caller-selected setting unavailable to the selected operation.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LimitError {
    /// QSL recognizes the setting, but this operation does not expose it.
    #[error("limit setting `{}` is unavailable for the `run` operation", setting.name())]
    OperationUnavailable {
        /// The recognized setting absent from the operation's bounds.
        setting: SettingName,
    },
}

/// Named, non-negative settings supplied with repeatable `--limit` options.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LimitOverrides(Vec<(SettingName, u64)>);

impl LimitOverrides {
    /// Parse operands with QSL's canonical setting grammar.
    pub fn parse(operands: &[String]) -> Result<Self, UsageRefusal> {
        qsl_foundation::setting::parse_operands(operands.iter().map(String::as_str)).map(Self)
    }

    /// Apply settings exposed by the selected `run` operation.
    pub fn apply_to(&self, limits: &mut SpineLimits, call: &mut Call) -> Result<(), LimitError> {
        self.apply_parts(limits, None, &mut call.accounting)
    }

    /// Apply settings to the actual stage, observation and accounting limits of a clause run.
    pub fn apply_to_clause(&self, request: &mut ClauseRunRequest) -> Result<(), LimitError> {
        self.apply_parts(
            &mut request.limits,
            Some(&mut request.observation_limits),
            &mut request.accounting,
        )
    }

    fn apply_parts(
        &self,
        limits: &mut SpineLimits,
        mut observation: Option<&mut ObservationLimits>,
        accounting: &mut quire_exact::ScalarLimits,
    ) -> Result<(), LimitError> {
        let mut available: Vec<Setting> = limits
            .bounds()
            .into_iter()
            .map(|(setting, _)| setting)
            .collect();
        if let Some(observation) = observation.as_deref() {
            available.extend(observation.bounds().into_iter().map(|(setting, _)| setting));
        }
        let mut updated_accounting = AccountingLimits(*accounting);
        let mut caller_limits = CallerLimits {
            spine: *limits,
            ..CallerLimits::default()
        };
        if let Some(observation) = observation.as_deref() {
            caller_limits.observation = *observation;
        }
        for (setting, value) in &self.0 {
            match setting {
                SettingName::Stage(setting) => {
                    if !available.contains(setting) {
                        return Err(LimitError::OperationUnavailable {
                            setting: SettingName::Stage(*setting),
                        });
                    }
                    // `bounds()` and `set_bound()` are the paired QSL API for
                    // the selected operation; an exposed setting must be writable.
                    let updated = caller_limits.set_bound(*setting, *value);
                    debug_assert!(updated, "QSL bound listed by bounds() must be settable");
                    if !updated {
                        return Err(LimitError::OperationUnavailable {
                            setting: SettingName::Stage(*setting),
                        });
                    }
                }
                SettingName::Accounting(setting) => {
                    updated_accounting.set_bound(*setting, *value);
                }
            }
        }
        *limits = caller_limits.spine;
        if let Some(observation) = observation.as_mut() {
            **observation = caller_limits.observation;
        }
        *accounting = updated_accounting.0;
        Ok(())
    }

    /// The parsed QSL settings, for caller inspection.
    pub fn values(&self) -> &[(SettingName, u64)] {
        &self.0
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use qsl_foundation::{
        AccountingSetting, Setting, SettingLimits, SettingName, UsageCause, UsageRefusal,
    };
    use qsl_replay::spine::{default_accounting, Call, SpineLimits};
    use qsl_replay::AccountingLimits;

    use super::{LimitError, LimitOverrides};

    fn call() -> Call {
        Call {
            function: "run".to_owned(),
            arguments: Vec::new(),
            accounting: default_accounting(100),
            supplied: quire_semantic_value::supplied_stop::SuppliedLimits::default(),
            backend: Default::default(),
        }
    }

    fn clause_request() -> qsl_replay::spine::ClauseRunRequest {
        qsl_replay::spine::ClauseRunRequest {
            source: qsl_replay::spine::ClauseRunSource::Program {
                identity: qsl_foundation::SourceIdentity::new("issuer", "unit"),
                path: "unit.qsl".to_owned(),
                bytes: Vec::new(),
            },
            package_name: "unit".to_owned(),
            packages: Default::default(),
            dependencies: Default::default(),
            snapshots: Default::default(),
            invocations: Default::default(),
            selection: qsl_replay::spine::ClauseRunSelection::Function {
                name: "yes".to_owned(),
                arguments: Vec::new(),
                snapshot: qsl_semantics::model::observation::DocumentRef {
                    authority: "issuer".to_owned(),
                    identity: "snapshot".to_owned(),
                },
            },
            limits: SpineLimits::default(),
            observation_limits: qsl_replay::ObservationLimits::default(),
            accounting: default_accounting(17),
        }
    }

    #[ix_trace_rs::trace("TC-010", "FR-003-AC-4")]
    #[test]
    fn clause_limits_apply_to_the_actual_observation_and_accounting_bounds() {
        let mut request = clause_request();
        let original = request.observation_limits;
        let (setting, value) = original.bounds()[0];
        let value = value.saturating_sub(1);
        let limits = LimitOverrides::parse(&[
            format!("{}={value}", setting.name()),
            "accounting.work_units=3".to_owned(),
        ])
        .expect("parse owner setting names");
        limits
            .apply_to_clause(&mut request)
            .expect("clause exposes observation and accounting");
        assert_eq!(
            request
                .observation_limits
                .bounds()
                .into_iter()
                .find(|(name, _)| *name == setting),
            Some((setting, value))
        );
        assert_eq!(
            AccountingLimits(request.accounting)
                .bounds()
                .into_iter()
                .find(|(name, _)| *name == qsl_foundation::AccountingSetting::WorkUnits)
                .map(|(_, value)| value),
            Some(3)
        );
        assert_eq!(request.limits, SpineLimits::default());
    }

    #[ix_trace_rs::trace("TC-010", "FR-003-AC-5")]
    #[test]
    fn invalid_clause_limit_is_atomic_across_all_actual_bounds() {
        let mut request = clause_request();
        let original_spine = request.limits;
        let original_observation = request.observation_limits;
        let original_accounting = request.accounting;
        let limits = LimitOverrides::parse(&[
            "s1.input_bytes=1".to_owned(),
            "accounting.work_units=0".to_owned(),
            "explore.states=1".to_owned(),
        ])
        .expect("all settings are recognized");
        assert!(limits.apply_to_clause(&mut request).is_err());
        assert_eq!(request.limits, original_spine);
        assert_eq!(request.observation_limits, original_observation);
        assert_eq!(request.accounting, original_accounting);
    }

    /// Trace: FR-003-AC-1, FR-003-AC-4, FR-003-AC-6.
    #[test]
    fn one_limit_override_preserves_other_defaults() {
        let mut before = SpineLimits::default();
        let prior_tokens = 17_u64;
        assert!(SettingLimits::set_bound(
            &mut before,
            Setting::S1Tokens,
            prior_tokens
        ));
        let mut limits = before;
        let mut call = call();
        let before_accounting = AccountingLimits(call.accounting).bounds();
        let override_ = LimitOverrides::parse(&["s1.input_bytes=5".to_owned()])
            .expect("QSL parses its setting operand");
        assert_eq!(
            override_.values(),
            &[(SettingName::Stage(Setting::S1InputBytes), 5)]
        );
        override_
            .apply_to(&mut limits, &mut call)
            .expect("setting is exposed");
        assert_eq!(limits.source.source_bytes, 5);
        assert_eq!(
            AccountingLimits(call.accounting).bounds(),
            before_accounting
        );
        assert_eq!(limits.source.tokens, before.source.tokens);
        assert_eq!(limits.checking, before.checking);
        assert_eq!(limits.model, before.model);
    }

    /// Trace: FR-003-AC-5, applying a later unavailable setting is atomic.
    #[test]
    fn unavailable_setting_leaves_all_prior_bounds_unchanged() {
        let before = SpineLimits::default();
        let mut limits = before;
        let mut call = call();
        let before_accounting = AccountingLimits(call.accounting).bounds();
        let overrides = LimitOverrides::parse(&[
            "s1.input_bytes=5".to_owned(),
            "explore.states=11".to_owned(),
        ])
        .expect("both setting names are recognized by QSL");

        assert_eq!(
            overrides.apply_to(&mut limits, &mut call),
            Err(LimitError::OperationUnavailable {
                setting: SettingName::Stage(Setting::ExploreStates),
            })
        );
        assert_eq!(limits, before);
        assert_eq!(
            AccountingLimits(call.accounting).bounds(),
            before_accounting
        );
    }

    /// Trace: FR-003-AC-1, FR-003-AC-4, FR-003-AC-6.
    #[test]
    fn every_exposed_setting_updates_only_its_qsl_bound() {
        let defaults = SpineLimits::default();
        let default_bounds = defaults.bounds();
        for (setting, original) in default_bounds.iter().copied() {
            let requested = if original < u64::MAX {
                original + 1
            } else {
                original - 1
            };
            let operand = format!("{}={requested}", setting.name());
            let overrides = LimitOverrides::parse(&[operand]).expect("QSL parses exposed name");
            let mut effective = defaults;
            let mut call = call();
            overrides
                .apply_to(&mut effective, &mut call)
                .expect("QSL exposes setting to run");
            for (observed, bound) in effective.bounds() {
                assert_eq!(
                    bound,
                    if observed == setting {
                        requested
                    } else {
                        default_bounds
                            .iter()
                            .copied()
                            .find(|(default_setting, _)| *default_setting == observed)
                            .map(|(_, default_bound)| default_bound)
                            .expect("default has the same exposed setting")
                    },
                    "only {setting} should change"
                );
            }
        }
    }

    /// Trace: FR-003-AC-2, FR-003-AC-7, FR-003-AC-8.
    #[test]
    fn qsl_parser_refuses_unknown_malformed_and_duplicate_operands() {
        assert!(matches!(
            LimitOverrides::parse(&["normalization.work_units=1".to_owned()]),
            Err(UsageRefusal {
                cause: UsageCause::UnknownSetting,
                ..
            })
        ));
        assert!(matches!(
            LimitOverrides::parse(&["s1.nodes=ten".to_owned()]),
            Err(UsageRefusal {
                cause: UsageCause::NotAnInteger,
                ..
            })
        ));
        assert!(matches!(
            LimitOverrides::parse(&["s1.nodes=1".to_owned(), "s1.nodes=2".to_owned()]),
            Err(UsageRefusal {
                cause: UsageCause::Repeated,
                ..
            })
        ));
    }

    /// Trace: FR-003-AC-5, recognized settings outside run bounds are distinct refusals.
    #[test]
    fn recognized_but_unavailable_setting_has_operation_specific_error() {
        let overrides = LimitOverrides::parse(&["explore.states=11".to_owned()])
            .expect("QSL recognizes exploration settings");
        let mut limits = SpineLimits::default();
        let mut call = call();
        assert_eq!(
            overrides.apply_to(&mut limits, &mut call),
            Err(LimitError::OperationUnavailable {
                setting: SettingName::Stage(Setting::ExploreStates),
            })
        );
    }

    /// Trace: FR-003-AC-1, FR-003-AC-6, QSL applies the exposed setting.
    #[test]
    fn s3_nodes_changes_only_the_checking_node_ceiling() {
        let overrides =
            LimitOverrides::parse(&["s3.nodes=7".to_owned()]).expect("QSL recognizes the setting");
        let mut limits = SpineLimits::default();
        let mut call = call();
        let original_checking = limits.checking;
        overrides
            .apply_to(&mut limits, &mut call)
            .expect("QSL exposes the typed node setting");
        assert_eq!(limits.checking, original_checking.with_nodes(7));
    }

    /// Trace: FR-003-AC-9, FR-003-AC-10. QSL-named accounting settings update only the call.
    #[test]
    fn each_named_accounting_setting_updates_only_call_accounting() {
        let defaults = call().accounting;
        let default_bounds = AccountingLimits(defaults).bounds();
        for setting in AccountingSetting::ALL {
            let operand = format!("{}=7", setting.name());
            let overrides = LimitOverrides::parse(&[operand]).expect("QSL parses counter name");
            assert_eq!(overrides.values(), &[(SettingName::Accounting(setting), 7)]);
            let mut limits = SpineLimits::default();
            let before_limits = limits;
            let mut call = call();
            overrides
                .apply_to(&mut limits, &mut call)
                .expect("accounting budgets are available to execute");
            assert_eq!(limits, before_limits);
            for (observed, bound) in AccountingLimits(call.accounting).bounds() {
                let expected = if observed == setting {
                    7
                } else {
                    default_bounds
                        .iter()
                        .find_map(|(default_setting, bound)| {
                            (*default_setting == observed).then_some(*bound)
                        })
                        .expect("QSL exposes every accounting default")
                };
                assert_eq!(bound, expected, "only {setting} should change");
            }
        }
    }

    /// Trace: FR-003-AC-1, typed defaults are inherited when no override is supplied.
    #[test]
    fn an_empty_limit_set_is_a_noop() {
        let mut limits = SpineLimits::default();
        let mut call = call();
        let expected = limits;
        let overrides = LimitOverrides::parse(&[]).expect("empty set");
        overrides
            .apply_to(&mut limits, &mut call)
            .expect("empty set applies");
        assert_eq!(limits, expected);
        assert!(overrides.values().is_empty());
    }
}
