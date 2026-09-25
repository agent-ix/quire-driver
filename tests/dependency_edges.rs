// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001-AC-1 (TC-005): the driver's dependency edges match ADR-011 §6.1's
//! driver row and §7.1's crate graph, read from `cargo metadata`'s resolved
//! graph. QSL packages are identified by their source URL, and normal and
//! build edges both count.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use serde_json::Value;

/// The source of every package built from the QSL repository.
const QSL_SOURCE: &str = "git+https://github.com/agent-ix/quire-spec-language";
/// The QSL root crate, which is not a layer crate (ADR-011 §6.1 crate map).
const QSL_ROOT: &str = "quire-spec-language";

struct Package {
    name: String,
    source: Option<String>,
}

impl Package {
    fn is_qsl(&self) -> bool {
        self.source
            .as_deref()
            .is_some_and(|source| source.starts_with(QSL_SOURCE))
    }
}

struct Graph {
    packages: BTreeMap<String, Package>,
    /// Package id -> the package ids it reaches by a normal or build edge.
    deps: BTreeMap<String, Vec<String>>,
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
        let packages = metadata["packages"]
            .as_array()
            .expect("packages")
            .iter()
            .map(|p| {
                (
                    p["id"].as_str().expect("id").to_owned(),
                    Package {
                        name: p["name"].as_str().expect("name").to_owned(),
                        source: p["source"].as_str().map(str::to_owned),
                    },
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
                    .filter(|dep| {
                        dep["dep_kinds"]
                            .as_array()
                            .expect("dep_kinds")
                            .iter()
                            .any(|k| k["kind"].is_null() || k["kind"] == "build")
                    })
                    .map(|dep| dep["pkg"].as_str().expect("pkg").to_owned())
                    .collect();
                (node["id"].as_str().expect("node id").to_owned(), edges)
            })
            .collect();
        let root = metadata["resolve"]["root"]
            .as_str()
            .expect("root")
            .to_owned();
        Self {
            packages,
            deps,
            root,
        }
    }

    fn package(&self, id: &str) -> &Package {
        &self.packages[id]
    }

    fn deps_of(&self, id: &str) -> impl Iterator<Item = &Package> {
        self.deps[id].iter().map(|dep| self.package(dep))
    }

    fn ids_named(&self, name: &str) -> Vec<&str> {
        self.packages
            .iter()
            .filter(|(_, p)| p.name == name)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

#[test]
fn tc_005_driver_edges_match_the_adr_011_allow_list() {
    let graph = Graph::load();
    assert_eq!(graph.package(&graph.root).name, "quire-driver");

    // §6.1 driver row: the driver's normal and build edges are QSL layer
    // crates, CG and one error-derive crate. Never the QSL root crate.
    let direct: BTreeSet<&str> = graph
        .deps_of(&graph.root)
        .map(|p| p.name.as_str())
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
    let driver_qsl: Vec<&Package> = graph.deps_of(&graph.root).filter(|p| p.is_qsl()).collect();
    assert_eq!(driver_qsl.len(), 5, "five QSL edges, by source");
    assert!(driver_qsl.iter().all(|p| p.name != QSL_ROOT));

    // Nothing in this lock depends on the driver.
    assert_eq!(graph.ids_named("quire-driver").len(), 1);
    for id in graph.deps.keys() {
        assert!(
            graph.deps_of(id).all(|p| p.name != "quire-driver"),
            "`{}` depends on quire-driver",
            graph.package(id).name
        );
    }

    // §7.1 / FB-05: CG's one normal or build edge into QSL is `qsl-replay`.
    let cg = graph.ids_named("quire-contract-codegen");
    assert_eq!(cg.len(), 1, "one CG revision");
    let cg_qsl: Vec<&Package> = graph.deps_of(cg[0]).filter(|p| p.is_qsl()).collect();
    let cg_qsl_names: Vec<&str> = cg_qsl.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(cg_qsl_names, ["qsl-replay"]);

    // The driver's QSL crates and CG's `qsl-replay` come from one QSL
    // revision, and each QSL package other than the root resolves once. The
    // root crate arrives transitively through IR at another revision
    // (Cargo.toml's revision note), which this test does not gate.
    let revisions: BTreeSet<&str> = driver_qsl
        .iter()
        .chain(&cg_qsl)
        .map(|p| p.source.as_deref().expect("git source"))
        .collect();
    assert_eq!(revisions.len(), 1, "{revisions:?}");
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for package in graph.packages.values().filter(|p| p.is_qsl()) {
        *counts.entry(package.name.as_str()).or_default() += 1;
    }
    for (name, count) in counts {
        assert!(
            name == QSL_ROOT || count == 1,
            "`{name}` resolves to {count} revisions"
        );
    }
}
