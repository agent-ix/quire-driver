// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001-AC-1 (TC-005): the driver's dependency edges match ADR-011 §6.1's
//! driver row and §7.1's crate graph, read from `cargo metadata`'s resolved
//! graph.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use serde_json::Value;

/// The QSL layer crates of ADR-011 §6.1's crate map, plus K.
const QSL_LAYER_CRATES: &[&str] = &[
    "qsl-foundation",
    "qsl-cst",
    "qsl-source",
    "qsl-forms",
    "qsl-semantics",
    "qsl-package",
    "qsl-eval",
    "qsl-route",
    "qsl-replay",
    "quire-exact",
];

struct Graph {
    /// Package id -> name.
    names: BTreeMap<String, String>,
    /// Package id -> (dependency package id, has a normal edge).
    deps: BTreeMap<String, Vec<(String, bool)>>,
    root: String,
}

impl Graph {
    fn load() -> Self {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
        let output = Command::new(cargo)
            .args(["metadata", "--format-version", "1", "--locked", "--offline"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("cargo metadata runs");
        assert!(
            output.status.success(),
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: Value = serde_json::from_slice(&output.stdout).expect("metadata is JSON");
        let names = metadata["packages"]
            .as_array()
            .expect("packages")
            .iter()
            .map(|p| {
                (
                    p["id"].as_str().expect("id").to_owned(),
                    p["name"].as_str().expect("name").to_owned(),
                )
            })
            .collect();
        let deps = metadata["resolve"]["nodes"]
            .as_array()
            .expect("resolve nodes")
            .iter()
            .map(|node| {
                let edges = node["deps"]
                    .as_array()
                    .expect("deps")
                    .iter()
                    .map(|dep| {
                        let normal = dep["dep_kinds"]
                            .as_array()
                            .expect("dep_kinds")
                            .iter()
                            .any(|k| k["kind"].is_null());
                        (dep["pkg"].as_str().expect("pkg").to_owned(), normal)
                    })
                    .collect();
                (node["id"].as_str().expect("node id").to_owned(), edges)
            })
            .collect();
        let root = metadata["resolve"]["root"]
            .as_str()
            .expect("root")
            .to_owned();
        Self { names, deps, root }
    }

    fn name(&self, id: &str) -> &str {
        &self.names[id]
    }

    fn normal_deps(&self, id: &str) -> BTreeSet<&str> {
        self.deps[id]
            .iter()
            .filter(|(_, normal)| *normal)
            .map(|(dep, _)| dep.as_str())
            .collect()
    }

    fn ids_named(&self, name: &str) -> Vec<&str> {
        self.names
            .iter()
            .filter(|(_, n)| n.as_str() == name)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

#[test]
fn tc_005_driver_edges_match_the_adr_011_allow_list() {
    let graph = Graph::load();
    assert_eq!(graph.name(&graph.root), "quire-driver");

    // §6.1 driver row: the driver's normal edges are the QSL layer crates
    // and CG, and one error-derive crate. Never the QSL root crate.
    let direct: BTreeSet<&str> = graph
        .normal_deps(&graph.root)
        .into_iter()
        .map(|id| graph.name(id))
        .collect();
    let expected: BTreeSet<&str> = [
        "qsl-foundation",
        "qsl-package",
        "qsl-replay",
        "qsl-route",
        "qsl-semantics",
        "quire-contract-codegen",
        "thiserror",
    ]
    .into_iter()
    .collect();
    assert_eq!(direct, expected);
    for name in &direct {
        assert!(
            QSL_LAYER_CRATES.contains(name)
                || *name == "quire-contract-codegen"
                || *name == "thiserror",
            "`{name}` is not on the driver row's allow-list"
        );
    }

    // Nothing depends on the driver.
    for (id, edges) in &graph.deps {
        assert!(
            edges.iter().all(|(dep, _)| *dep != graph.root),
            "`{}` depends on quire-driver",
            graph.name(id)
        );
    }

    // §7.1 / FB-05: CG's one normal QSL-workspace edge is `qsl-replay`.
    let cg = graph.ids_named("quire-contract-codegen");
    assert_eq!(cg.len(), 1, "one CG revision");
    let cg_qsl: Vec<&str> = graph
        .normal_deps(cg[0])
        .into_iter()
        .map(|id| graph.name(id))
        .filter(|name| QSL_LAYER_CRATES.contains(name) || *name == "quire-spec-language")
        .collect();
    assert_eq!(cg_qsl, ["qsl-replay"]);

    // One revision of each QSL layer crate, so the driver's QSL crates and
    // CG's `qsl-replay` are the same packages.
    for name in QSL_LAYER_CRATES {
        let ids = graph.ids_named(name);
        assert!(
            ids.len() <= 1,
            "`{name}` resolves to {} revisions: {ids:?}",
            ids.len()
        );
    }
    let driver_replay: Vec<&str> = graph
        .normal_deps(&graph.root)
        .into_iter()
        .filter(|id| graph.name(id) == "qsl-replay")
        .collect();
    let cg_replay: Vec<&str> = graph
        .normal_deps(cg[0])
        .into_iter()
        .filter(|id| graph.name(id) == "qsl-replay")
        .collect();
    assert_eq!(driver_replay, cg_replay);
}
