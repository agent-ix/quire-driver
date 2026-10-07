// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resolve and apply FR-003's caller-selected QSL limit settings through QSL.

use qsl_foundation::{Setting, SettingLimits, UsageRefusal};
use qsl_replay::{spine::SpineLimits, CallerLimits};

/// A recognized caller-selected setting unavailable to the selected operation.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LimitError {
    /// QSL recognizes the setting, but this operation does not expose it.
    #[error("limit setting `{setting}` is unavailable for the `run` operation")]
    OperationUnavailable {
        /// The recognized setting absent from the operation's bounds.
        setting: Setting,
    },
}

/// Named, non-negative settings supplied with repeatable `--limit` options.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LimitOverrides(Vec<(Setting, u64)>);

impl LimitOverrides {
    /// Parse operands with QSL's canonical setting grammar.
    pub fn parse(operands: &[String]) -> Result<Self, UsageRefusal> {
        qsl_foundation::setting::parse_operands(operands.iter().map(String::as_str)).map(Self)
    }

    /// Apply settings exposed by the selected `run` operation.
    pub fn apply_to(&self, limits: &mut SpineLimits) -> Result<(), LimitError> {
        let available: Vec<Setting> = limits
            .bounds()
            .into_iter()
            .map(|(setting, _)| setting)
            .collect();
        let mut caller_limits = CallerLimits {
            spine: *limits,
            ..CallerLimits::default()
        };
        for (setting, value) in &self.0 {
            if !available.contains(setting) {
                return Err(LimitError::OperationUnavailable { setting: *setting });
            }
            // `bounds()` and `set_bound()` are the paired QSL API for the
            // selected operation; an exposed setting must be writable.
            let updated = caller_limits.set_bound(*setting, *value);
            debug_assert!(updated, "QSL bound listed by bounds() must be settable");
            if !updated {
                return Err(LimitError::OperationUnavailable { setting: *setting });
            }
        }
        *limits = caller_limits.spine;
        Ok(())
    }

    /// The parsed QSL settings, for caller inspection.
    pub fn values(&self) -> &[(Setting, u64)] {
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
    use qsl_foundation::{Setting, SettingLimits, UsageCause, UsageRefusal};
    use qsl_replay::spine::SpineLimits;

    use super::{LimitError, LimitOverrides};

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
        let override_ = LimitOverrides::parse(&["s1.input_bytes=5".to_owned()])
            .expect("QSL parses its setting operand");
        assert_eq!(override_.values(), &[(Setting::S1InputBytes, 5)]);
        override_.apply_to(&mut limits).expect("setting is exposed");
        assert_eq!(limits.source.source_bytes, 5);
        assert_eq!(limits.source.tokens, before.source.tokens);
        assert_eq!(limits.checking, before.checking);
        assert_eq!(limits.model, before.model);
    }

    /// Trace: FR-003-AC-5, applying a later unavailable setting is atomic.
    #[test]
    fn unavailable_setting_leaves_all_prior_bounds_unchanged() {
        let before = SpineLimits::default();
        let mut limits = before;
        let overrides = LimitOverrides::parse(&[
            "s1.input_bytes=5".to_owned(),
            "explore.states=11".to_owned(),
        ])
        .expect("both setting names are recognized by QSL");

        assert_eq!(
            overrides.apply_to(&mut limits),
            Err(LimitError::OperationUnavailable {
                setting: Setting::ExploreStates,
            })
        );
        assert_eq!(limits, before);
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
            overrides
                .apply_to(&mut effective)
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
            LimitOverrides::parse(&["work_units=1".to_owned()]),
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
        assert_eq!(
            overrides.apply_to(&mut limits),
            Err(LimitError::OperationUnavailable {
                setting: Setting::ExploreStates,
            })
        );
    }

    /// Trace: FR-003-AC-1, FR-003-AC-6, QSL applies the exposed setting.
    #[test]
    fn s3_nodes_changes_only_the_checking_node_ceiling() {
        let overrides =
            LimitOverrides::parse(&["s3.nodes=7".to_owned()]).expect("QSL recognizes the setting");
        let mut limits = SpineLimits::default();
        let original_checking = limits.checking;
        overrides
            .apply_to(&mut limits)
            .expect("QSL exposes the typed node setting");
        assert_eq!(limits.checking, original_checking.with_nodes(7));
    }

    /// Trace: FR-003-AC-1, typed defaults are inherited when no override is supplied.
    #[test]
    fn an_empty_limit_set_is_a_noop() {
        let mut limits = SpineLimits::default();
        let expected = limits;
        let overrides = LimitOverrides::parse(&[]).expect("empty set");
        overrides.apply_to(&mut limits).expect("empty set applies");
        assert_eq!(limits, expected);
        assert!(overrides.values().is_empty());
    }
}
