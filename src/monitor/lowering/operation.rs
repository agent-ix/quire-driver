// SPDX-License-Identifier: AGPL-3.0-or-later
//! Closed direct scalar mappings; source admission and accounting stay with the caller.

use quire_contract_model::{ApplicationOperator, CheckedOperationRead};
use quire_contract_runtime::monitor::{ExpressionId, Operation};
use quire_exact::{OrderingOperator, ValueType};

/// Map only the exact admitted plain operation/shape declared by FR-017.
/// `None` asks the caller to retain the original located Unsupported payload;
/// it never grants permission to evaluate or rewrite the source operation.
pub(super) fn map_application(
    operator: &ApplicationOperator,
    source: CheckedOperationRead<'_>,
    arguments: &[ExpressionId],
    operands: &[ValueType],
    result: &ValueType,
) -> Option<Operation> {
    if source.mode.is_some()
        || source.member.is_some()
        || source.execution_point.is_some()
        || !source.laws.is_empty()
        || !source.constraints.is_empty()
        || !source.leaves.is_empty()
    {
        return None;
    }
    match (operator, arguments, operands, result) {
        (
            ApplicationOperator::Binary,
            [left, right],
            [ValueType::Integer, ValueType::Integer],
            ValueType::Integer,
        ) => match source.identity {
            "quire.op.integer.add" => Some(Operation::IntegerAdd {
                left: *left,
                right: *right,
            }),
            "quire.op.integer.sub" => Some(Operation::IntegerSubtract {
                left: *left,
                right: *right,
            }),
            "quire.op.integer.mul" => Some(Operation::IntegerMultiply {
                left: *left,
                right: *right,
            }),
            _ => None,
        },
        (ApplicationOperator::Unary, [value], [ValueType::Integer], ValueType::Integer)
            if source.identity == "quire.op.integer.negate" =>
        {
            Some(Operation::IntegerNegate { value: *value })
        }
        (
            ApplicationOperator::Binary,
            [left, right],
            [ValueType::Integer, ValueType::Integer],
            ValueType::Boolean,
        ) => {
            let operator = match source.identity {
                "quire.op.integer.lt" => OrderingOperator::Less,
                "quire.op.integer.le" => OrderingOperator::LessOrEqual,
                "quire.op.integer.gt" => OrderingOperator::Greater,
                "quire.op.integer.ge" => OrderingOperator::GreaterOrEqual,
                _ => return None,
            };
            Some(Operation::IntegerOrder {
                operator,
                left: *left,
                right: *right,
            })
        }
        (ApplicationOperator::Unary, [value], [ValueType::Boolean], ValueType::Boolean)
            if source.identity == "quire.op.boolean.not" =>
        {
            Some(Operation::BooleanNot { value: *value })
        }
        (
            ApplicationOperator::Binary,
            [left, right],
            [ValueType::Boolean, ValueType::Boolean],
            ValueType::Boolean,
        ) => match source.identity {
            "quire.op.boolean.and" => Some(Operation::BooleanAnd {
                left: *left,
                right: *right,
            }),
            "quire.op.boolean.or" => Some(Operation::BooleanOr {
                left: *left,
                right: *right,
            }),
            _ => None,
        },
        _ => None,
    }
}
