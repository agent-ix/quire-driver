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

    fn reachable_from(&self, start: &str) -> BTreeSet<&str> {
        let mut seen = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(id) = pending.pop() {
            for next in &self.deps[id] {
                if seen.insert(next.as_str()) {
                    pending.push(next);
                }
            }
        }
        seen
    }
}

/// FR-001-AC-1, FR-001-AC-11 and FR-012-AC-14 (TC-005)
#[test]
fn tc_005_driver_edges_match_the_adr_011_allow_list() {
    let graph = Graph::load();
    assert_eq!(graph.package(&graph.root).name, "quire-driver");

    // §6.1 driver row: the driver's normal and build edges are QSL layer
    // crates, quire-exact (QSL's caller-owned Cancel API), CG, IR's model
    // crate (E5's reader and `CheckedNodeId`) and one error-derive crate.
    // Never the QSL root crate. `serde_json` is a
    // dev-dependency (this crate's own tests use it), so it is not one of
    // these normal/build edges.
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
        "quire-exact",
        "quire-contract-codegen",
        "quire-contract-model",
        "thiserror",
    ]
    .into_iter()
    .collect();
    assert_eq!(direct, expected);
    assert!(
        graph
            .reachable_from(&graph.root)
            .iter()
            .all(|id| graph.package(id).name != "quire-cache"),
        "the quire-driver transitive dependency closure reaches quire-cache"
    );
    let driver_qsl: Vec<&Package> = graph.deps_of(&graph.root).filter(|p| p.is_qsl()).collect();
    assert_eq!(driver_qsl.len(), 5, "five QSL edges, by source");
    assert!(driver_qsl.iter().all(|p| p.name != QSL_ROOT));

    // The driver core has no incoming edges from upstream QSL, IR or CG.
    // Downstream components may depend on its cache interface.
    assert_eq!(graph.ids_named("quire-driver").len(), 1);
    for id in graph.deps.keys() {
        let package = graph.package(id);
        let upstream = package.is_qsl()
            || matches!(
                package.name.as_str(),
                "quire-contract-codegen" | "quire-contract-ir" | "quire-contract-model"
            );
        if upstream {
            assert!(
                graph.deps_of(id).all(|p| p.name != "quire-driver"),
                "upstream `{}` depends on quire-driver",
                package.name
            );
        }
    }

    let cache = graph.ids_named("quire-cache");
    if !cache.is_empty() {
        assert_eq!(cache.len(), 1, "one downstream cache implementation");
        assert!(
            graph.deps_of(cache[0]).any(|p| p.name == "quire-driver"),
            "quire-cache must depend on the driver-owned cache interface"
        );
    }

    // §7.1 / FB-05: CG's one normal or build edge into QSL is `qsl-replay`.
    let cg = graph.ids_named("quire-contract-codegen");
    assert_eq!(cg.len(), 1, "one CG revision");
    let cg_qsl: Vec<&Package> = graph.deps_of(cg[0]).filter(|p| p.is_qsl()).collect();
    let cg_qsl_names: Vec<&str> = cg_qsl.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(cg_qsl_names, ["qsl-replay"]);

    // The driver's model crate is the one CG's IR crate re-exports, so a
    // `CheckedPackageV2` the driver reads is the type CG generates from.
    let driver_model: Vec<&String> = graph.deps[&graph.root]
        .iter()
        .filter(|id| graph.package(id).name == "quire-contract-model")
        .collect();
    assert_eq!(driver_model.len(), 1);
    let cg_ir: Vec<&String> = graph.deps[cg[0]]
        .iter()
        .filter(|id| graph.package(id).name == "quire-contract-ir")
        .collect();
    assert_eq!(cg_ir.len(), 1, "CG has one IR edge");
    assert!(
        graph.deps[cg_ir[0]].contains(driver_model[0]),
        "the driver's `quire-contract-model` ({}) is not CG's IR's",
        driver_model[0]
    );

    // One copy of `quire-contract-model` across the whole graph, so the
    // driver's direct edge, CG's IR crate and QSL's `qsl-package` all resolve
    // the same package id and no IR type crosses between copies.
    assert_eq!(
        graph.ids_named("quire-contract-model").len(),
        1,
        "one quire-contract-model copy"
    );
}
