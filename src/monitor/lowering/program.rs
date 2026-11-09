// SPDX-License-Identifier: AGPL-3.0-or-later
//! Lowering of the selected monitor's entry bodies and the helper functions
//! they reach, into the runtime's flat function and expression arenas.
//!
//! Every operation is mapped from the admitted operation identity and the
//! admitted node facts; nothing is evaluated and no meaning is supplied by a
//! callback. An operation outside the closed set is a located `Unsupported`.

use std::collections::BTreeMap;

use quire_contract_model::{
    CheckedBindingNameKind, CheckedNodeId, CheckedOperationMember, CheckedPackageV2, CheckedTerm,
    LiteralKind, SelectedMonitorTypeClosure,
};
use quire_contract_runtime::monitor::{
    Entries, Expression, ExpressionId, Function, FunctionId, LocalId, Operation,
};
use quire_exact::{Cancel, Integer, MemberId, MemberOwner, ValueType};
use quire_semantic_value::declaration::CheckedEquality;

use super::{key, operation, Budget, Error, MonitorPreparationLimits};
use crate::monitor::ClosureFacts;
use crate::monitor_refusal::unsupported_at;

/// A conservative upper bound on the native stack one lowering level
/// (`lower`, `application` or `value`, `operands`) occupies.
const LOWER_FRAME_BYTES: usize = 4096;

/// The lowered arenas and the resolved entry addresses.
pub struct LoweredBodies {
    /// The four entries, as function addresses.
    pub entries: Entries,
    /// The retained function closure.
    pub functions: Vec<Function>,
    /// The retained expression closure.
    pub expressions: Vec<Expression>,
}

struct Pending<'a> {
    id: FunctionId,
    node: &'a CheckedNodeId,
    entry_key: Option<&'a CheckedNodeId>,
    entry_body: Option<&'a CheckedTerm>,
}

/// What a lowered expression reads, for sharing and binder context.
struct Info {
    closed: bool,
    free: Vec<usize>,
}

struct Scope {
    name: String,
    level: u64,
    local: LocalId,
}

struct Frame<'a> {
    id: FunctionId,
    parameters: Vec<&'a CheckedNodeId>,
    locals: usize,
    scope: Vec<Scope>,
    memo: BTreeMap<&'a CheckedNodeId, ExpressionId>,
}

struct Lowerer<'a, 'b> {
    package: &'a CheckedPackageV2,
    closure: &'b SelectedMonitorTypeClosure<'a>,
    facts: &'b ClosureFacts<'b>,
    budget: Budget<'b>,
    callees: Vec<&'a CheckedNodeId>,
    queue: Vec<Pending<'a>>,
    expressions: Vec<Expression>,
    info: Vec<Info>,
    shared: BTreeMap<&'a CheckedNodeId, ExpressionId>,
    shared_inline: BTreeMap<(&'a CheckedNodeId, usize), ExpressionId>,
}

/// Lower the four monitor entries and every helper function they reach.
pub fn lower_entries<'a>(
    package: &'a CheckedPackageV2,
    closure: &SelectedMonitorTypeClosure<'a>,
    facts: &ClosureFacts<'_>,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<LoweredBodies, Error> {
    let plan = closure.plan();
    let mut lowerer = Lowerer {
        package,
        closure,
        facts,
        budget: Budget::new(limits, cancel),
        callees: Vec::new(),
        queue: Vec::new(),
        expressions: Vec::new(),
        info: Vec::new(),
        shared: BTreeMap::new(),
        shared_inline: BTreeMap::new(),
    };
    let entries = [
        plan.init(),
        plan.on_event(),
        plan.on_watermark(),
        plan.on_gap(),
    ];
    for (index, entry) in entries.iter().enumerate() {
        lowerer.callees.push(entry.node_id());
        lowerer.queue.push(Pending {
            id: FunctionId(index),
            node: entry.node_id(),
            entry_key: Some(entry.structural_key()),
            entry_body: Some(entry.body()),
        });
    }
    let mut functions: Vec<Option<Function>> = Vec::new();
    while let Some(pending) = lowerer.queue.pop() {
        let function = lowerer.function(&pending)?;
        if functions.len() <= pending.id.0 {
            functions.resize_with(pending.id.0 + 1, || None);
        }
        functions[pending.id.0] = Some(function);
    }
    let functions = functions
        .into_iter()
        .map(|function| {
            function.ok_or_else(|| Error::InvalidAdmittedEntry {
                node: Box::new(plan.node_id().clone()),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LoweredBodies {
        entries: Entries {
            init: FunctionId(0),
            on_event: FunctionId(1),
            on_watermark: FunctionId(2),
            on_gap: FunctionId(3),
        },
        functions,
        expressions: lowerer.expressions,
    })
}

fn binding_kind<'t>(
    term: &'t CheckedTerm,
) -> Option<(CheckedBindingNameKind<'t>, &'t CheckedTerm)> {
    match term {
        CheckedTerm::Binding { name, value } => Some((name.kind(), value)),
        _ => None,
    }
}

impl<'a, 'b> Lowerer<'a, 'b> {
    fn invalid(&self, node: &CheckedNodeId) -> Error {
        Error::InvalidAdmittedScalar {
            node: Box::new(node.clone()),
        }
    }

    fn unsupported(&self, node: &CheckedNodeId) -> Error {
        let Some(kind) = self.package.node_kind(node) else {
            return self.invalid(node);
        };
        match unsupported_at(self.package, node, &[], kind, None) {
            Ok(payload) => Error::Unsupported(Box::new(payload)),
            Err(error) => Error::Internal(error),
        }
    }

    fn value_type(&mut self, node: &CheckedNodeId) -> Result<ValueType, Error> {
        self.budget.charge(1)?;
        self.closure
            .exact_types()
            .iter()
            .find(|projection| projection.node() == node)
            .map(|projection| projection.value_type().clone())
            .ok_or_else(|| self.invalid(node))
    }

    fn semantic_type(&self, node: &CheckedNodeId) -> Result<&'a CheckedNodeId, Error> {
        self.graph_node(node).map(|found| &found.semantic_type)
    }

    fn graph_node(
        &self,
        node: &CheckedNodeId,
    ) -> Result<&'a quire_contract_model::CheckedSemanticNodeV2, Error> {
        self.package
            .resolved_graph()
            .nodes
            .iter()
            .find(|found| &found.node_id == node)
            .ok_or_else(|| self.invalid(node))
    }

    fn callee(&mut self, node: &'a CheckedNodeId) -> Result<FunctionId, Error> {
        self.budget.charge(1)?;
        if let Some(index) = self.callees.iter().position(|known| *known == node) {
            return Ok(FunctionId(index));
        }
        let id = FunctionId(self.callees.len());
        self.callees.push(node);
        self.queue.push(Pending {
            id,
            node,
            entry_key: None,
            entry_body: None,
        });
        Ok(id)
    }

    fn function(&mut self, pending: &Pending<'a>) -> Result<Function, Error> {
        let (parameters, body) = match pending.entry_body {
            Some(term) => self.entry_shape(pending.node, term)?,
            None => {
                let read = self
                    .package
                    .source_callable(pending.node, self.budget.remaining())?;
                self.budget.charge(read.work())?;
                let callable = read
                    .callable()
                    .ok_or_else(|| self.unsupported(pending.node))?;
                if callable.context.is_some() || callable.frame.is_some() {
                    return Err(self.unsupported(pending.node));
                }
                let parameters: Vec<&'a CheckedNodeId> = callable
                    .parameters
                    .iter()
                    .map(|parameter| self.graph_node(parameter).map(|found| &found.node_id))
                    .collect::<Result<_, _>>()?;
                let body = &self.graph_node(&callable.body)?.node_id;
                (parameters, body)
            }
        };
        let mut types = Vec::with_capacity(parameters.len());
        for parameter in &parameters {
            let semantic = self.semantic_type(parameter)?;
            types.push(self.value_type(semantic)?);
        }
        let mut frame = Frame {
            id: pending.id,
            parameters,
            locals: 0,
            scope: Vec::new(),
            memo: BTreeMap::new(),
        };
        let body = self.lower(&mut frame, body)?;
        let original = pending
            .entry_key
            .or_else(|| self.package.structural_key(pending.node))
            .ok_or_else(|| self.invalid(pending.node))?;
        let key = key::own_key(original, &mut self.budget)?;
        Ok(Function {
            key,
            parameters: types,
            result: self.expressions[body.0].value_type.clone(),
            body,
        })
    }

    #[allow(clippy::type_complexity)]
    fn entry_shape(
        &self,
        node: &'a CheckedNodeId,
        term: &'a CheckedTerm,
    ) -> Result<(Vec<&'a CheckedNodeId>, &'a CheckedNodeId), Error> {
        let CheckedTerm::Aggregate { members } = term else {
            return Err(Error::InvalidAdmittedEntry {
                node: Box::new(node.clone()),
            });
        };
        let (mut parameters, mut body) = (None, None);
        for member in members {
            match binding_kind(member) {
                Some((CheckedBindingNameKind::Parameters, CheckedTerm::Aggregate { members })) => {
                    let mut list = Vec::with_capacity(members.len());
                    for member in members {
                        let CheckedTerm::Reference { target } = member else {
                            return Err(self.unsupported(node));
                        };
                        list.push(target);
                    }
                    parameters = Some(list);
                }
                Some((CheckedBindingNameKind::Body, CheckedTerm::Reference { target })) => {
                    body = Some(target);
                }
                _ => {}
            }
        }
        match (parameters, body) {
            (Some(parameters), Some(body)) => Ok((parameters, body)),
            _ => Err(Error::InvalidAdmittedEntry {
                node: Box::new(node.clone()),
            }),
        }
    }

    /// A closed expression reads no argument or local, so any function may
    /// share it; the runtime requires such an expression to exist once.
    /// `free` lists the binder slots it reads that it does not itself bind.
    fn analyze(&self, operation: &Operation) -> Info {
        let mut closed = true;
        let mut free: Vec<usize> = Vec::new();
        let read =
            |id: &ExpressionId, bound: &[usize], free: &mut Vec<usize>, closed: &mut bool| {
                let info = &self.info[id.0];
                *closed &= info.closed;
                free.extend(
                    info.free
                        .iter()
                        .copied()
                        .filter(|slot| !bound.contains(slot)),
                );
            };
        match operation {
            Operation::Literal(_) => {}
            Operation::Argument { .. } => closed = false,
            Operation::Local(local) => {
                closed = false;
                free.push(local.position);
            }
            Operation::Call { arguments: ids, .. }
            | Operation::Tuple { positions: ids }
            | Operation::Union { positions: ids, .. }
            | Operation::Sequence { elements: ids } => {
                ids.iter()
                    .for_each(|id| read(id, &[], &mut free, &mut closed));
            }
            Operation::Record { fields } => {
                fields
                    .iter()
                    .for_each(|(_, id)| read(id, &[], &mut free, &mut closed));
            }
            Operation::SequenceSize { value }
            | Operation::SequenceFlatten { value }
            | Operation::IntegerNegate { value }
            | Operation::BooleanNot { value }
            | Operation::TupleElement { value, .. }
            | Operation::UnionIs { value, .. }
            | Operation::UnionPayload { value, .. }
            | Operation::RecordField { value, .. } => read(value, &[], &mut free, &mut closed),
            Operation::IntegerAdd { left, right }
            | Operation::IntegerSubtract { left, right }
            | Operation::IntegerMultiply { left, right }
            | Operation::IntegerOrder { left, right, .. }
            | Operation::BooleanAnd { left, right }
            | Operation::BooleanOr { left, right }
            | Operation::Equality { left, right, .. } => {
                read(left, &[], &mut free, &mut closed);
                read(right, &[], &mut free, &mut closed);
            }
            Operation::If {
                condition,
                when_true,
                when_false,
            } => {
                for id in [condition, when_true, when_false] {
                    read(id, &[], &mut free, &mut closed);
                }
            }
            Operation::Let {
                binding,
                value,
                body,
            } => {
                closed = false;
                read(value, &[], &mut free, &mut closed);
                read(body, &[binding.position], &mut free, &mut closed);
            }
            Operation::SequenceFold {
                sequence,
                initial,
                accumulator,
                item,
                body,
            } => {
                closed = false;
                read(sequence, &[], &mut free, &mut closed);
                read(initial, &[], &mut free, &mut closed);
                read(
                    body,
                    &[accumulator.position, item.position],
                    &mut free,
                    &mut closed,
                );
            }
            Operation::SequenceMap {
                sequence,
                item,
                body,
            }
            | Operation::SequenceFilter {
                sequence,
                item,
                predicate: body,
            } => {
                closed = false;
                read(sequence, &[], &mut free, &mut closed);
                read(body, &[item.position], &mut free, &mut closed);
            }
        }
        free.sort_unstable();
        free.dedup();
        Info { closed, free }
    }

    fn push(
        &mut self,
        node: &'a CheckedNodeId,
        value_type: ValueType,
        operation: Operation,
    ) -> Result<ExpressionId, Error> {
        self.push_at(node, Vec::new(), value_type, operation)
    }

    fn push_at(
        &mut self,
        node: &'a CheckedNodeId,
        body_path: Vec<usize>,
        value_type: ValueType,
        operation: Operation,
    ) -> Result<ExpressionId, Error> {
        let original = self
            .package
            .structural_key(node)
            .ok_or_else(|| self.invalid(node))?;
        let key = key::own_key(original, &mut self.budget)?;
        let id = ExpressionId(self.expressions.len());
        let info = self.analyze(&operation);
        // The structural key does not carry binder identity, so an expression
        // reading binder slots is located by those slots after a marker that
        // no inline operand position can equal.
        let body_path = if body_path.is_empty() && !info.free.is_empty() {
            std::iter::once(usize::MAX)
                .chain(info.free.iter().copied())
                .collect()
        } else {
            body_path
        };
        self.info.push(info);
        self.expressions.push(Expression {
            key,
            body_path,
            value_type,
            operation,
        });
        Ok(id)
    }

    /// Lower operand terms: a reference is its own node; an inline literal
    /// is located by its position in the parent's argument list.
    fn operands(
        &mut self,
        frame: &mut Frame<'a>,
        node: &'a CheckedNodeId,
        terms: &'a [CheckedTerm],
        first: usize,
    ) -> Result<Vec<ExpressionId>, Error> {
        let mut ids = Vec::with_capacity(terms.len());
        for (offset, term) in terms.iter().enumerate() {
            let id = match term {
                CheckedTerm::Reference { target } => self.lower(frame, target)?,
                CheckedTerm::Literal {
                    type_id,
                    value_kind,
                    value,
                } => {
                    self.budget.charge(1)?;
                    let parent = self
                        .package
                        .structural_key(node)
                        .ok_or_else(|| self.invalid(node))?;
                    if let Some(done) = self.shared_inline.get(&(parent, first + offset)) {
                        ids.push(*done);
                        continue;
                    }
                    let value_type = self.value_type(type_id)?;
                    let literal = self.literal(node, value_kind, value)?;
                    let id = self.push_at(
                        node,
                        vec![first + offset],
                        value_type,
                        Operation::Literal(literal),
                    )?;
                    self.shared_inline.insert((parent, first + offset), id);
                    id
                }
                _ => return Err(self.unsupported(node)),
            };
            ids.push(id);
        }
        Ok(ids)
    }

    fn literal(
        &self,
        node: &CheckedNodeId,
        kind: &LiteralKind,
        value: &quire_contract_model::CheckedJson,
    ) -> Result<quire_exact::Value, Error> {
        Ok(match kind {
            LiteralKind::Integer => {
                let text = value
                    .as_str()
                    .or_else(|| value.number_text())
                    .ok_or_else(|| self.invalid(node))?;
                quire_exact::Value::Integer(
                    text.parse::<Integer>().map_err(|_| self.invalid(node))?,
                )
            }
            LiteralKind::Boolean => {
                quire_exact::Value::Boolean(value.as_bool().ok_or_else(|| self.invalid(node))?)
            }
            LiteralKind::String => quire_exact::Value::String(
                value.as_str().ok_or_else(|| self.invalid(node))?.to_owned(),
            ),
            _ => return Err(self.unsupported(node)),
        })
    }

    fn lower(
        &mut self,
        frame: &mut Frame<'a>,
        node: &'a CheckedNodeId,
    ) -> Result<ExpressionId, Error> {
        self.budget.charge(1)?;
        let structural = self
            .package
            .structural_key(node)
            .ok_or_else(|| self.invalid(node))?;
        if let Some(done) = frame
            .memo
            .get(structural)
            .or_else(|| self.shared.get(structural))
        {
            return Ok(*done);
        }
        let term = self.package.term(node).ok_or_else(|| self.invalid(node))?;
        let id = match term {
            CheckedTerm::Literal {
                type_id,
                value_kind,
                value,
            } => {
                let value_type = self.value_type(type_id)?;
                let literal = self.literal(node, value_kind, value)?;
                self.push(node, value_type, Operation::Literal(literal))?
            }
            CheckedTerm::Application {
                operation,
                arguments,
                result_type,
                ..
            } => self.application(frame, node, operation, arguments, result_type)?,
            CheckedTerm::Aggregate { members } => {
                if let Some((name, level)) = parameter_shape(members) {
                    self.variable(frame, node, &name, level)?
                } else {
                    self.value(frame, node, members)?
                }
            }
            _ => return Err(self.unsupported(node)),
        };
        frame.memo.insert(structural, id);
        if self.info[id.0].closed {
            self.shared.insert(structural, id);
        }
        Ok(id)
    }

    fn variable(
        &mut self,
        frame: &mut Frame<'a>,
        node: &'a CheckedNodeId,
        name: &str,
        level: u64,
    ) -> Result<ExpressionId, Error> {
        let semantic = self.semantic_type(node)?;
        let value_type = self.value_type(semantic)?;
        if let Some(position) = frame.parameters.iter().position(|known| *known == node) {
            return self.push(
                node,
                value_type,
                Operation::Argument {
                    function: frame.id,
                    position,
                },
            );
        }
        let local = frame
            .scope
            .iter()
            .rev()
            .find(|scope| scope.name == name && scope.level == level)
            .map(|scope| scope.local)
            .ok_or_else(|| self.invalid(node))?;
        self.push(node, value_type, Operation::Local(local))
    }

    /// Close the binders above `depth`; expressions that read their slots are
    /// valid only inside, so they leave the function's sharing table.
    fn leave(&self, frame: &mut Frame<'a>, depth: usize) {
        let gone: Vec<usize> = frame.scope[depth..]
            .iter()
            .map(|scope| scope.local.position)
            .collect();
        frame.scope.truncate(depth);
        frame
            .memo
            .retain(|_, id| !self.info[id.0].free.iter().any(|slot| gone.contains(slot)));
    }

    fn bind(&mut self, frame: &mut Frame<'a>, name: &str) -> LocalId {
        let local = LocalId {
            function: frame.id,
            position: frame.locals,
        };
        frame.locals += 1;
        let level = (frame.parameters.len() + frame.scope.len()) as u64;
        frame.scope.push(Scope {
            name: name.to_owned(),
            level,
            local,
        });
        local
    }

    fn value(
        &mut self,
        frame: &mut Frame<'a>,
        node: &'a CheckedNodeId,
        members: &'a [CheckedTerm],
    ) -> Result<ExpressionId, Error> {
        let semantic = self.semantic_type(node)?;
        let value_type = self.value_type(semantic)?;
        let facts = self.facts;
        if let Some(row) = facts
            .source_records
            .iter()
            .find(|row| &row.structural_node == semantic)
        {
            // QSL's generated records declare every member required and emit
            // every member (checked against the genuine plan in
            // `monitor_closure.rs`), so a missing binding is an invariant
            // fault, never an absent optional member.
            let mut fields = Vec::with_capacity(row.members.len());
            for declared in &row.members {
                let value = members
                    .iter()
                    .find_map(|member| match binding_kind(member) {
                        Some((
                            CheckedBindingNameKind::Other(name),
                            CheckedTerm::Reference { target },
                        )) if name == declared.admitted_name.as_ref() => Some(target),
                        _ => None,
                    })
                    .ok_or_else(|| self.invalid(node))?;
                let member = MemberId::new(
                    MemberOwner::Node(row.record_key.clone()),
                    declared.admitted_name.as_ref(),
                )
                .map_err(|_| self.invalid(node))?;
                fields.push((member, self.lower(frame, value)?));
            }
            return self.push(node, value_type, Operation::Record { fields });
        }
        if facts
            .source_tuples
            .iter()
            .any(|row| &row.structural_node == semantic)
        {
            let positions = self.operands(frame, node, members, 0)?;
            return self.push(node, value_type, Operation::Tuple { positions });
        }
        if let Some(row) = facts
            .source_unions
            .iter()
            .find(|row| &row.structural_node == semantic)
        {
            let [member] = members else {
                return Err(self.invalid(node));
            };
            let Some((
                CheckedBindingNameKind::Other(case),
                CheckedTerm::Aggregate { members: payload },
            )) = binding_kind(member)
            else {
                return Err(self.invalid(node));
            };
            let declared = row
                .members
                .iter()
                .find(|declared| declared.member.identifier() == case)
                .ok_or_else(|| self.invalid(node))?;
            let positions = self.operands(frame, node, payload, 0)?;
            return self.push(
                node,
                value_type,
                Operation::Union {
                    variant: declared.member.variant().clone(),
                    positions,
                },
            );
        }
        Err(self.unsupported(node))
    }

    fn application(
        &mut self,
        frame: &mut Frame<'a>,
        node: &'a CheckedNodeId,
        operation: &'a quire_contract_model::CheckedApplicationOperation,
        arguments: &'a [CheckedTerm],
        result_type: &'a CheckedNodeId,
    ) -> Result<ExpressionId, Error> {
        let read = operation.read().map_err(Error::OperationRead)?;
        self.budget.charge(7)?;
        let value_type = self.value_type(result_type)?;
        let operator = match self.package.term(node) {
            Some(CheckedTerm::Application { operator, .. }) => operator,
            _ => return Err(self.invalid(node)),
        };
        let lowered = match read.identity {
            "quire.op.function.call" => {
                let Some((CheckedTerm::Reference { target: callee }, rest)) =
                    arguments.split_first()
                else {
                    return Err(self.unsupported(node));
                };
                let function = self.callee(callee)?;
                let operands = self.operands(frame, node, rest, 1)?;
                Operation::Call {
                    function,
                    arguments: operands,
                }
            }
            "quire.op.control.if" => {
                let [condition, when_true, when_false] =
                    self.operands(frame, node, arguments, 0)?[..]
                else {
                    return Err(self.unsupported(node));
                };
                Operation::If {
                    condition,
                    when_true,
                    when_false,
                }
            }
            "quire.op.control.let" => {
                let [binding, CheckedTerm::Reference { target: body }] = arguments else {
                    return Err(self.unsupported(node));
                };
                let Some((
                    CheckedBindingNameKind::Other(name),
                    CheckedTerm::Reference { target: init },
                )) = binding_kind(binding)
                else {
                    return Err(self.unsupported(node));
                };
                let value = self.lower(frame, init)?;
                let depth = frame.scope.len();
                let binding = self.bind(frame, name);
                let body = self.lower(frame, body)?;
                self.leave(frame, depth);
                Operation::Let {
                    binding,
                    value,
                    body,
                }
            }
            "quire.op.collection.sequence" => {
                let elements = self.operands(frame, node, arguments, 0)?;
                Operation::Sequence { elements }
            }
            "quire.op.collection.flatten" | "quire.op.collection.size" => {
                let [value] = self.operands(frame, node, arguments, 0)?[..] else {
                    return Err(self.unsupported(node));
                };
                if read.identity.ends_with("flatten") {
                    Operation::SequenceFlatten { value }
                } else {
                    Operation::SequenceSize { value }
                }
            }
            "quire.op.collection.fold" => {
                let [CheckedTerm::Reference { target: sequence }, binding, CheckedTerm::Reference { target: initial }] =
                    arguments
                else {
                    return Err(self.unsupported(node));
                };
                let Some((
                    CheckedBindingNameKind::Other(accumulator),
                    CheckedTerm::Aggregate { members },
                )) = binding_kind(binding)
                else {
                    return Err(self.unsupported(node));
                };
                let [inner] = &members[..] else {
                    return Err(self.unsupported(node));
                };
                let Some((
                    CheckedBindingNameKind::Other(item),
                    CheckedTerm::Reference { target: body },
                )) = binding_kind(inner)
                else {
                    return Err(self.unsupported(node));
                };
                let sequence = self.lower(frame, sequence)?;
                let initial = self.lower(frame, initial)?;
                let depth = frame.scope.len();
                let accumulator = self.bind(frame, accumulator);
                let item = self.bind(frame, item);
                let body = self.lower(frame, body)?;
                self.leave(frame, depth);
                Operation::SequenceFold {
                    sequence,
                    initial,
                    accumulator,
                    item,
                    body,
                }
            }
            "quire.op.collection.map" | "quire.op.collection.filter" => {
                let [CheckedTerm::Reference { target: sequence }, binding] = arguments else {
                    return Err(self.unsupported(node));
                };
                let Some((
                    CheckedBindingNameKind::Other(item),
                    CheckedTerm::Reference { target: body },
                )) = binding_kind(binding)
                else {
                    return Err(self.unsupported(node));
                };
                let sequence = self.lower(frame, sequence)?;
                let depth = frame.scope.len();
                let item = self.bind(frame, item);
                let body = self.lower(frame, body)?;
                self.leave(frame, depth);
                if read.identity.ends_with("map") {
                    Operation::SequenceMap {
                        sequence,
                        item,
                        body,
                    }
                } else {
                    Operation::SequenceFilter {
                        sequence,
                        item,
                        predicate: body,
                    }
                }
            }
            "quire.op.record.project" => {
                let [value] = self.operands(frame, node, arguments, 0)?[..] else {
                    return Err(self.unsupported(node));
                };
                match read.member {
                    Some(CheckedOperationMember::Field {
                        declaration, name, ..
                    }) => {
                        let row = self
                            .facts
                            .source_records
                            .iter()
                            .find(|row| &row.structural_node == declaration)
                            .ok_or_else(|| self.invalid(node))?;
                        let member =
                            MemberId::new(MemberOwner::Node(row.record_key.clone()), name.as_ref())
                                .map_err(|_| self.invalid(node))?;
                        Operation::RecordField { value, member }
                    }
                    Some(CheckedOperationMember::Position { position, .. }) => {
                        let position = position
                            .to_string()
                            .parse::<usize>()
                            .map_err(|_| self.invalid(node))?;
                        Operation::TupleElement { value, position }
                    }
                    _ => return Err(self.unsupported(node)),
                }
            }
            "quire.op.integer.eq"
            | "quire.op.integer.ne"
            | "quire.op.boolean.eq"
            | "quire.op.boolean.ne" => {
                let [left, right] = self.operands(frame, node, arguments, 0)?[..] else {
                    return Err(self.unsupported(node));
                };
                let decision = self
                    .closure
                    .equalities()
                    .iter()
                    .find(|equality| &equality.id.application == node)
                    .ok_or_else(|| self.unsupported(node))?;
                Operation::Equality {
                    left,
                    right,
                    checked: Box::new(CheckedEquality::from_admitted_decision(&decision.decision)),
                }
            }
            _ => {
                let ids = self.operands(frame, node, arguments, 0)?;
                let types: Vec<_> = ids
                    .iter()
                    .map(|id| self.expressions[id.0].value_type.clone())
                    .collect();
                operation::map_application(operator, read, &ids, &types, &value_type)
                    .ok_or_else(|| self.unsupported(node))?
            }
        };
        self.push(node, value_type, lowered)
    }
}

/// A variable node is the closed two-binding aggregate `{name, level}`.
fn parameter_shape(members: &[CheckedTerm]) -> Option<(String, u64)> {
    let [name, level] = members else { return None };
    let (CheckedBindingNameKind::Other("name"), CheckedTerm::Literal { value: name, .. }) =
        binding_kind(name)?
    else {
        return None;
    };
    let (CheckedBindingNameKind::Other("level"), CheckedTerm::Literal { value: level, .. }) =
        binding_kind(level)?
    else {
        return None;
    };
    Some((
        name.as_str()?.to_owned(),
        level
            .as_str()
            .or_else(|| level.number_text())?
            .parse()
            .ok()?,
    ))
}
