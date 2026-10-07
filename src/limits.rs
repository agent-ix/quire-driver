// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parse and apply FR-003's caller-selected QSL limit settings.

use std::collections::BTreeMap;

use qsl_replay::spine::{Call, SpineLimits};

/// A malformed or currently unavailable caller-selected limit.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LimitError {
    /// The operand did not use the required `name=decimal` form.
    #[error(
        "invalid limit operand `{operand}`; expected name=value with a non-negative decimal value"
    )]
    MalformedOperand {
        /// The original option value.
        operand: String,
    },
    /// The setting is not a supported QSL field.
    #[error("unknown or unsupported limit setting `{name}` in `{operand}`")]
    UnknownSetting {
        /// The unrecognized setting name.
        name: String,
        /// The original option value.
        operand: String,
    },
    /// The same setting was supplied more than once.
    #[error("duplicate limit setting `{name}` in `{operand}`")]
    DuplicateSetting {
        /// The setting supplied more than once.
        name: String,
        /// The duplicate option value.
        operand: String,
    },
    /// The decimal operand is larger than the supported wire integer.
    #[error("limit operand `{operand}` exceeds u64")]
    ValueOutOfRange {
        /// The original option value that exceeds the supported range.
        operand: String,
    },
    /// The value cannot fit the platform-sized field exposed by QSL.
    #[error("limit setting `{name}` value {value} is too large on this platform")]
    PlatformValueOverflow {
        /// The platform-sized QSL setting.
        name: String,
        /// The requested value that cannot fit this platform's `usize`.
        value: u64,
    },
}

/// Named, non-negative settings supplied with repeatable `--limit` options.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LimitOverrides(BTreeMap<String, u64>);

impl LimitOverrides {
    /// Parse unique `<setting>=<non-negative decimal>` operands.
    pub fn parse(operands: &[String]) -> Result<Self, LimitError> {
        let mut values = BTreeMap::new();
        for operand in operands {
            let (name, raw_value) =
                operand
                    .split_once('=')
                    .ok_or_else(|| LimitError::MalformedOperand {
                        operand: operand.clone(),
                    })?;
            if name.is_empty()
                || raw_value.is_empty()
                || !raw_value.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(LimitError::MalformedOperand {
                    operand: operand.clone(),
                });
            }
            if !is_supported(name) {
                return Err(LimitError::UnknownSetting {
                    name: name.to_owned(),
                    operand: operand.clone(),
                });
            }
            let value = raw_value
                .parse::<u64>()
                .map_err(|_| LimitError::ValueOutOfRange {
                    operand: operand.clone(),
                })?;
            if values.insert(name.to_owned(), value).is_some() {
                return Err(LimitError::DuplicateSetting {
                    name: name.to_owned(),
                    operand: operand.clone(),
                });
            }
        }
        Ok(Self(values))
    }

    /// Apply the selected settings to QSL's lifecycle and interpreter limits.
    ///
    /// The names and field ownership follow QSL FR-255. Depth settings are
    /// intentionally absent; depth is not a caller limit.
    pub fn apply_to(&self, limits: &mut SpineLimits, call: &mut Call) -> Result<(), LimitError> {
        for (name, value) in &self.0 {
            let value = *value;
            match name.as_str() {
                "s1.input_bytes" => limits.source.source_bytes = as_usize(name, value)?,
                "s1.tokens" => limits.source.tokens = as_usize(name, value)?,
                "s1.nodes" => limits.source.nodes = as_usize(name, value)?,
                "s1.work_units" => limits.source.work_units = as_usize(name, value)?,
                "s3.nodes" => limits.checking = limits.checking.with_nodes(value),
                "s3.input_bytes" => {
                    limits.checking = limits.checking.with_input_bytes(value);
                }
                "s3.work_units" => {
                    limits.checking = limits.checking.with_work_budget(value);
                }
                "normalization.declaration_records" => limits.model.declaration_records = value,
                "normalization.derivation_facts" => limits.model.derivation_facts = value,
                "normalization.effective_declarations" => {
                    limits.model.effective_declarations = value;
                }
                "normalization.dispatch_candidates" => {
                    limits.model.dispatch_candidates = value;
                }
                "normalization.hashed_bytes" => limits.model.hashed_bytes = value,
                "normalization.work_units" => limits.model.work_units = value,
                "normalization.ancestor_steps" => limits.model.ancestor_steps = value,
                "normalization.family_steps" => limits.model.family_steps = value,
                "types.ancestor_steps" => limits.assembly.environment.ancestor_steps = value,
                "types.work_units" => limits.assembly.environment.work_units = value,
                "integer_bits" => call.accounting.integer_bits = value,
                "decimal_digits" => call.accounting.decimal_digits = value,
                "scale_expansion" => call.accounting.scale_expansion = value,
                "text_input_bytes" => call.accounting.text_input_bytes = value,
                "text_scalars" => call.accounting.text_scalars = value,
                "normalized_scalars" => call.accounting.normalized_scalars = value,
                "unit_edges" => call.accounting.unit_edges = value,
                "value_occurrences" => call.accounting.value_occurrences = value,
                "work_units" => call.accounting.work_units = value,
                "result_units" => call.accounting.result_units = value,
                _ => {
                    return Err(LimitError::UnknownSetting {
                        name: name.to_owned(),
                        operand: format!("{name}={value}"),
                    })
                }
            }
        }
        Ok(())
    }

    /// The parsed settings, for evidence and caller inspection.
    pub fn values(&self) -> &BTreeMap<String, u64> {
        &self.0
    }
}

fn as_usize(name: &str, value: u64) -> Result<usize, LimitError> {
    usize::try_from(value).map_err(|_| LimitError::PlatformValueOverflow {
        name: name.to_owned(),
        value,
    })
}

fn is_supported(name: &str) -> bool {
    matches!(
        name,
        "s1.input_bytes"
            | "s1.tokens"
            | "s1.nodes"
            | "s1.work_units"
            | "s3.nodes"
            | "s3.input_bytes"
            | "s3.work_units"
            | "normalization.declaration_records"
            | "normalization.derivation_facts"
            | "normalization.effective_declarations"
            | "normalization.dispatch_candidates"
            | "normalization.hashed_bytes"
            | "normalization.work_units"
            | "normalization.ancestor_steps"
            | "normalization.family_steps"
            | "types.ancestor_steps"
            | "types.work_units"
            | "integer_bits"
            | "decimal_digits"
            | "scale_expansion"
            | "text_input_bytes"
            | "text_scalars"
            | "normalized_scalars"
            | "unit_edges"
            | "value_occurrences"
            | "work_units"
            | "result_units"
    )
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use qsl_replay::spine::{default_accounting, Call, SpineLimits};

    use super::{LimitError, LimitOverrides};

    fn call() -> Call {
        Call {
            function: "f".to_owned(),
            arguments: Vec::new(),
            accounting: default_accounting(19),
        }
    }

    /// Trace: FR-003-AC-1, one setting changes only its named field.
    #[test]
    fn one_limit_override_preserves_other_defaults() {
        let before = SpineLimits::default();
        let mut limits = before;
        let mut call = call();
        let override_ =
            LimitOverrides::parse(&["s1.input_bytes=5".to_owned()]).expect("valid setting");
        override_
            .apply_to(&mut limits, &mut call)
            .expect("applicable setting");
        assert_eq!(limits.source.source_bytes, 5);
        assert_eq!(limits.source.tokens, before.source.tokens);
        assert_eq!(limits.checking, before.checking);
        assert_eq!(limits.model, before.model);
        assert_eq!(call.accounting.work_units, 19);
    }

    /// Trace: FR-003-AC-2, malformed, duplicate, and unknown values are refused.
    #[test]
    fn rejects_unknown_malformed_and_duplicate_settings() {
        for operands in [
            vec!["s9.nodes=1".to_owned()],
            vec!["s1.nodes=ten".to_owned()],
            vec!["s1.nodes=-1".to_owned()],
            vec!["s1.nodes=1".to_owned(), "s1.nodes=2".to_owned()],
            vec!["dependencies.depth=2".to_owned()],
        ] {
            assert!(LimitOverrides::parse(&operands).is_err(), "{operands:?}");
        }
        assert!(matches!(
            LimitOverrides::parse(&["s1.nodes=1".to_owned(), "s1.nodes=2".to_owned()]),
            Err(LimitError::DuplicateSetting { .. })
        ));
        assert!(matches!(
            LimitOverrides::parse(&["s1.nodes=ten".to_owned()]),
            Err(LimitError::MalformedOperand { .. })
        ));
        assert!(matches!(
            LimitOverrides::parse(&["s9.nodes=1".to_owned()]),
            Err(LimitError::UnknownSetting { .. })
        ));
    }

    /// Trace: FR-003, values are passed unchanged to both the spine and interpreter.
    #[test]
    fn named_settings_reach_the_typed_operation_limits() {
        let mut limits = SpineLimits::default();
        let mut call = call();
        let overrides =
            LimitOverrides::parse(&["work_units=11".to_owned()]).expect("valid settings");
        overrides
            .apply_to(&mut limits, &mut call)
            .expect("apply settings");
        assert_eq!(call.accounting.work_units, 11);
    }

    /// Trace: FR-003-AC-1, s3.nodes changes only the checking node ceiling.
    #[test]
    fn s3_nodes_changes_only_the_checking_node_ceiling() {
        let overrides = LimitOverrides::parse(&["s3.nodes=7".to_owned()]).expect("known setting");
        let mut limits = SpineLimits::default();
        let original_checking = limits.checking;
        let mut call = call();
        overrides
            .apply_to(&mut limits, &mut call)
            .expect("QSL exposes the typed node builder");
        assert_eq!(limits.checking, original_checking.with_nodes(7));
    }

    /// Trace: FR-003, typed defaults are inherited when no override is supplied.
    #[test]
    fn an_empty_limit_set_is_a_noop() {
        let mut limits = SpineLimits::default();
        let expected = limits;
        let mut call = call();
        let original = call.accounting;
        let overrides = LimitOverrides::parse(&[]).expect("empty set");
        overrides
            .apply_to(&mut limits, &mut call)
            .expect("empty set applies");
        assert_eq!(limits, expected);
        assert_eq!(call.accounting, original);
        assert!(overrides.values().is_empty());
    }
}
