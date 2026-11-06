// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owning direct scalar lowering from selective admitted native callable facts.
//!
//! This component constructs native arenas, not a monitor Program or Layout.

mod key;
mod operation;
mod storage;

use quire_contract_model::{
    CheckedCallable, CheckedCallableIdentity, CheckedNodeId, CheckedPackageV2, CheckedTerm,
    NativeJoinBudget, SelectedExactIdentities,
};
use quire_contract_runtime::monitor::{Expression, ExpressionId, Function, FunctionId};
use quire_exact::{Cancel, ValueType};

use super::{budget::Budget, MonitorPreparationError as Error, MonitorPreparationLimits};
use crate::monitor_refusal::unsupported_at;

type NativeArenas = (Vec<FunctionId>, Vec<Function>, Vec<Expression>);

/// Lower the closed Integer/Boolean capability declared by FR-017.
///
/// Requested root order is retained and repeated roots share FunctionId.
/// Every returned arena owns its keys and types; no source borrow escapes.
/// The explicit allowance is independent of monitor admission. Native key
/// constructor reservations use the original owner hook. Input acquisition and
/// diagnostic clone allocation are not complete retained-storage qualification;
/// this is not full monitor preparation.
pub fn lower_scalar_functions(
    package: &CheckedPackageV2,
    roots: &[CheckedNodeId],
    identities: &SelectedExactIdentities,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<NativeArenas, Error> {
    let mut budget = Budget::new(limits, cancel);
    assemble(package, roots, identities, &mut budget)
}

// The full by-value monitor entry will share this worklist with a resumed
// budget, once its real Layout/environment/entry authority is available.
fn assemble(
    package: &CheckedPackageV2,
    roots: &[CheckedNodeId],
    identities: &SelectedExactIdentities,
    budget: &mut Budget<'_>,
) -> Result<NativeArenas, Error> {
    let mut requested = Vec::new();
    let mut functions = Vec::new();
    let mut expressions = Vec::new();
    let mut selected: Vec<&CheckedNodeId> = Vec::new();
    let mut scratch = 0;
    let mut retained = 0;
    budget.charge(0)?;
    for root in roots {
        budget.inspect(root)?;
        let mut existing = None;
        for (index, known) in selected.iter().enumerate() {
            budget.inspect(*known)?;
            if root == *known {
                existing = Some(FunctionId(index));
                break;
            }
        }
        let function = if let Some(existing) = existing {
            existing
        } else {
            let read = package.source_callable(root, budget.remaining())?;
            budget.charge(read.work())?;
            let Some(callable) = read.callable() else {
                return Err(unsupported(package, root)?);
            };
            if callable.context.is_some()
                || callable.frame.is_some()
                || !matches!(&callable.identity, CheckedCallableIdentity::Function(id) if id == root)
            {
                return Err(unsupported(package, root)?);
            }
            let function = FunctionId(functions.len());
            let result = scalar_type(package, root, identities, budget)?;
            let mut parameters = Vec::new();
            for parameter in &callable.parameters {
                let ty = scalar_type(package, parameter, identities, budget)?;
                storage::push(&mut parameters, ty, budget, &mut retained)?;
            }
            let body = lower_body(
                package,
                callable,
                function,
                identities,
                budget,
                &mut expressions,
                &mut retained,
            )?;
            if expressions[body.0].value_type != result {
                return Err(Error::InvalidAdmittedScalar {
                    node: Box::new(root.clone()),
                });
            }
            let original =
                package
                    .structural_key(root)
                    .ok_or_else(|| Error::InvalidAdmittedScalar {
                        node: Box::new(root.clone()),
                    })?;
            let key = key::own_key(original, budget)?;
            storage::push(
                &mut functions,
                Function {
                    key,
                    parameters,
                    result,
                    body,
                },
                budget,
                &mut retained,
            )?;
            storage::push(&mut selected, root, budget, &mut scratch)?;
            function
        };
        storage::push(&mut requested, function, budget, &mut retained)?;
    }
    drop(selected);
    budget.release_storage(scratch)?;
    budget.charge(0)?;
    Ok((requested, functions, expressions))
}

fn scalar_type(
    package: &CheckedPackageV2,
    node: &CheckedNodeId,
    identities: &SelectedExactIdentities,
    budget: &mut Budget<'_>,
) -> Result<ValueType, Error> {
    budget.inspect(node)?;
    let mut native_budget = NativeJoinBudget::new(budget.remaining(), budget.remaining_storage());
    let projection = package.project_exact_type(node, identities, &mut native_budget)?;
    budget.charge(projection.work())?;
    match projection.value_type() {
        ValueType::Integer => Ok(ValueType::Integer),
        ValueType::Boolean => Ok(ValueType::Boolean),
        _ => Err(unsupported(package, node)?),
    }
}

fn unsupported(package: &CheckedPackageV2, node: &CheckedNodeId) -> Result<Error, Error> {
    let kind = package
        .node_kind(node)
        .ok_or_else(|| Error::InvalidAdmittedScalar {
            node: Box::new(node.clone()),
        })?;
    unsupported_at(package, node, &[], kind, None)
        .map(|payload| Error::Unsupported(Box::new(payload)))
        .map_err(Error::Internal)
}

struct Memo<'a> {
    node: &'a CheckedNodeId,
    expression: Option<ExpressionId>,
}

enum Visit<'a> {
    Enter(&'a CheckedNodeId),
    Finish(usize),
    Alias(usize, &'a CheckedNodeId),
}

fn find(
    memo: &[Memo<'_>],
    node: &CheckedNodeId,
    budget: &mut Budget<'_>,
) -> Result<Option<usize>, Error> {
    for (index, entry) in memo.iter().enumerate() {
        budget.inspect(entry.node)?;
        budget.inspect(node)?;
        if entry.node == node {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn resolved(
    memo: &[Memo<'_>],
    node: &CheckedNodeId,
    budget: &mut Budget<'_>,
) -> Result<ExpressionId, Error> {
    find(memo, node, budget)?
        .and_then(|index| memo[index].expression)
        .ok_or_else(|| Error::InvalidAdmittedScalar {
            node: Box::new(node.clone()),
        })
}

fn lower_body(
    package: &CheckedPackageV2,
    callable: &CheckedCallable,
    owner: FunctionId,
    identities: &SelectedExactIdentities,
    budget: &mut Budget<'_>,
    expressions: &mut Vec<Expression>,
    retained: &mut usize,
) -> Result<ExpressionId, Error> {
    // Each successful expression in the initial set eventually reaches a
    // parameter: literals, zero-arity operations and calls are excluded.
    // Thus all successful entries have exactly this function's owner. A fresh
    // memo per function preserves genuine shared handles without key rewrites.
    let mut memo = Vec::new();
    let mut pending = Vec::new();
    let mut scratch = 0;
    storage::push(
        &mut pending,
        Visit::Enter(&callable.body),
        budget,
        &mut scratch,
    )?;
    while let Some(visit) = pending.pop() {
        budget.charge(1)?;
        match visit {
            Visit::Enter(node) => {
                if let Some(index) = find(&memo, node, budget)? {
                    if memo[index].expression.is_none() {
                        return Err(Error::InvalidAdmittedScalar {
                            node: Box::new(node.clone()),
                        });
                    }
                    continue;
                }
                let index = memo.len();
                storage::push(
                    &mut memo,
                    Memo {
                        node,
                        expression: None,
                    },
                    budget,
                    &mut scratch,
                )?;
                let mut position = None;
                for (at, parameter) in callable.parameters.iter().enumerate() {
                    budget.inspect(parameter)?;
                    budget.inspect(node)?;
                    if parameter == node {
                        position = Some(at);
                        break;
                    }
                }
                if let Some(position) = position {
                    let value_type = scalar_type(package, node, identities, budget)?;
                    let original = package.structural_key(node).ok_or_else(|| {
                        Error::InvalidAdmittedScalar {
                            node: Box::new(node.clone()),
                        }
                    })?;
                    let key = key::own_key(original, budget)?;
                    let id = ExpressionId(expressions.len());
                    storage::push(
                        expressions,
                        Expression {
                            key,
                            body_path: Vec::new(),
                            value_type,
                            operation: quire_contract_runtime::monitor::Operation::Argument {
                                function: owner,
                                position,
                            },
                        },
                        budget,
                        retained,
                    )?;
                    memo[index].expression = Some(id);
                    continue;
                }
                match package.term(node) {
                    Some(CheckedTerm::Reference { target }) => {
                        storage::push(
                            &mut pending,
                            Visit::Alias(index, target),
                            budget,
                            &mut scratch,
                        )?;
                        storage::push(&mut pending, Visit::Enter(target), budget, &mut scratch)?;
                    }
                    Some(CheckedTerm::Application {
                        operator,
                        operation,
                        arguments,
                        ..
                    }) => {
                        let record = operation.read().map_err(Error::OperationRead)?;
                        budget.inspect(record.identity)?;
                        budget.charge(7)?;
                        if !operation::accepts_header(operator, record, arguments.len()) {
                            return Err(unsupported(package, node)?);
                        }
                        // No accepted operation has more than two operands;
                        // refuse before building an unbounded temporary list.
                        if arguments.len() > 2 {
                            return Err(unsupported(package, node)?);
                        }
                        storage::push(&mut pending, Visit::Finish(index), budget, &mut scratch)?;
                        for argument in arguments.iter().rev() {
                            budget.charge(1)?;
                            let CheckedTerm::Reference { target } = argument else {
                                return Err(unsupported(package, node)?);
                            };
                            storage::push(
                                &mut pending,
                                Visit::Enter(target),
                                budget,
                                &mut scratch,
                            )?;
                        }
                    }
                    Some(_) => return Err(unsupported(package, node)?),
                    None => {
                        return Err(Error::InvalidAdmittedScalar {
                            node: Box::new(node.clone()),
                        })
                    }
                }
            }
            Visit::Alias(index, target) => {
                let target = resolved(&memo, target, budget)?;
                let node = memo[index].node;
                let value_type = scalar_type(package, node, identities, budget)?;
                if value_type != expressions[target.0].value_type {
                    return Err(Error::InvalidAdmittedScalar {
                        node: Box::new(node.clone()),
                    });
                }
                let original =
                    package
                        .structural_key(node)
                        .ok_or_else(|| Error::InvalidAdmittedScalar {
                            node: Box::new(node.clone()),
                        })?;
                let key = key::own_key(original, budget)?;
                // A source Reference root has its own original location. The
                // closed native operation is immutable and contains only IDs;
                // retaining it at that location preserves the reference's key
                // without inventing an RT wrapper or evaluating the operand.
                let operation = operation::copy_scalar_operation(&expressions[target.0].operation)
                    .ok_or_else(|| Error::InvalidAdmittedScalar {
                        node: Box::new(node.clone()),
                    })?;
                let id = ExpressionId(expressions.len());
                storage::push(
                    expressions,
                    Expression {
                        key,
                        body_path: Vec::new(),
                        value_type,
                        operation,
                    },
                    budget,
                    retained,
                )?;
                memo[index].expression = Some(id);
            }
            Visit::Finish(index) => {
                let node = memo[index].node;
                let Some(CheckedTerm::Application {
                    operator,
                    operation,
                    arguments,
                    ..
                }) = package.term(node)
                else {
                    return Err(Error::InvalidAdmittedScalar {
                        node: Box::new(node.clone()),
                    });
                };
                let mut ids = [ExpressionId(0); 2];
                let mut types = [ValueType::Integer, ValueType::Integer];
                for (position, argument) in arguments.iter().enumerate() {
                    let CheckedTerm::Reference { target } = argument else {
                        return Err(unsupported(package, node)?);
                    };
                    ids[position] = resolved(&memo, target, budget)?;
                    types[position] = expressions[ids[position].0].value_type.clone();
                }
                let value_type = scalar_type(package, node, identities, budget)?;
                let record = operation.read().map_err(Error::OperationRead)?;
                // Charge the actual retained metadata inspected by the closed
                // mapper; nonplain payloads are not silently discarded.
                budget.inspect(record.identity)?;
                budget.charge(7)?;
                let operation = operation::map_application(
                    operator,
                    record,
                    &ids[..arguments.len()],
                    &types[..arguments.len()],
                    &value_type,
                );
                let Some(operation) = operation else {
                    return Err(unsupported(package, node)?);
                };
                let original =
                    package
                        .structural_key(node)
                        .ok_or_else(|| Error::InvalidAdmittedScalar {
                            node: Box::new(node.clone()),
                        })?;
                let key = key::own_key(original, budget)?;
                let id = ExpressionId(expressions.len());
                storage::push(
                    expressions,
                    Expression {
                        key,
                        body_path: Vec::new(),
                        value_type,
                        operation,
                    },
                    budget,
                    retained,
                )?;
                memo[index].expression = Some(id);
            }
        }
    }
    let result = resolved(&memo, &callable.body, budget)?;
    drop(pending);
    drop(memo);
    budget.release_storage(scratch)?;
    Ok(result)
}
