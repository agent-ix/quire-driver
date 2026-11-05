// SPDX-License-Identifier: AGPL-3.0-or-later
//! Iterative conversion of original structural fields through the native key constructor.

use quire_contract_model::{CheckedNodeId, CheckedNodePathSegment};
use quire_exact::{NodeKey, PathSegment};

use super::{storage, Budget, Error};

struct Frame<'a> {
    source: &'a CheckedNodeId,
    next: usize,
    path: Vec<PathSegment>,
    path_storage: usize,
}

/// Driver-owned scratch, text and path storage is charged here. The current
/// native constructor's opaque internal reservations need its owner hook before
/// the complete retained-storage obligation can be qualified.
pub(super) fn own_key(source: &CheckedNodeId, budget: &mut Budget<'_>) -> Result<NodeKey, Error> {
    let mut frames = Vec::new();
    let mut frame_storage = 0;
    storage::push(
        &mut frames,
        Frame {
            source,
            next: 0,
            path: Vec::new(),
            path_storage: 0,
        },
        budget,
        &mut frame_storage,
    )?;
    while let Some(frame) = frames.last_mut() {
        budget.charge(1)?;
        if let Some(segment) = frame.source.path.get(frame.next) {
            frame.next += 1;
            match segment {
                CheckedNodePathSegment::Name(name) => {
                    let name = storage::text(name, budget)?;
                    storage::push(
                        &mut frame.path,
                        PathSegment::Text(name),
                        budget,
                        &mut frame.path_storage,
                    )?;
                }
                CheckedNodePathSegment::Node(child) => {
                    let child = child.as_ref();
                    storage::push(
                        &mut frames,
                        Frame {
                            source: child,
                            next: 0,
                            path: Vec::new(),
                            path_storage: 0,
                        },
                        budget,
                        &mut frame_storage,
                    )?;
                }
                CheckedNodePathSegment::Index(_) => {
                    return Err(Error::InvalidAdmittedScalar {
                        node: Box::new(source.clone()),
                    })
                }
            }
        } else {
            let Some(frame) = frames.pop() else {
                return Err(Error::InvalidAdmittedScalar {
                    node: Box::new(source.clone()),
                });
            };
            let package = storage::text(&frame.source.package, budget)?;
            budget.charge(8)?;
            let key =
                NodeKey::try_new(package, frame.path, frame.source.ordinal).map_err(Error::Key)?;
            // Native construction consumed and freed the input path vector;
            // its text and nested keys now belong to the returned native owner.
            budget.release_storage(frame.path_storage)?;
            if let Some(parent) = frames.last_mut() {
                storage::push(
                    &mut parent.path,
                    PathSegment::Node(key),
                    budget,
                    &mut parent.path_storage,
                )?;
            } else {
                drop(frames);
                budget.release_storage(frame_storage)?;
                return Ok(key);
            }
        }
    }
    Err(Error::InvalidAdmittedScalar {
        node: Box::new(source.clone()),
    })
}
