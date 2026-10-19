# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-driver's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The solid and dashed
boxes in the hero follow the verbs the `quire` binary handles today
(quire-cli/src/cli.rs) and the steps `drive` runs (src/drive.rs).
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: from source to verdict, what runs today and what is planned."""
    f = Figure(
        "hero",
        980,
        470,
        "The quire command takes QSL source through parse, check and compile to a checked package, "
        "and its run verb evaluates a function to an outcome with an exit code, all working today. "
        "The driver library also reads the package back, routes it, negotiates with backends and "
        "generates code, but no command exposes that yet. The prove and generate commands and "
        "their verdict are planned.",
    )
    top = 24
    f.text(0, 12, "QUIRE COMMAND · WORKS TODAY", "muted", "mono", SMALL)
    f.box(0, top, 140, 56, "code", "QSL source", ".native file")
    f.box(180, top, 110, 56, "logic", "parse", "quire parse")
    f.box(330, top, 110, 56, "logic", "check", "quire check")
    f.box(480, top, 130, 56, "logic", "compile", "quire compile")
    f.box(650, top, 190, 56, "model", "checked package", "checked-package/v2")
    for a, b in ((140, 177), (290, 327), (440, 477), (610, 647)):
        f.line([(a, top + 28), (b, top + 28)], "logic")

    run = 140
    f.box(180, run, 130, 56, "logic", "run", "interpreter")
    f.box(360, run, 220, 56, "evidence", "outcome", "result · exit code")
    f.line([(70, top + 56), (70, run + 28), (177, run + 28)], "code")
    f.line([(310, run + 28), (357, run + 28)], "logic")

    lib = 262
    f.text(0, lib - 12, "DRIVER LIBRARY · TESTED, NO COMMAND YET", "muted", "mono", SMALL)
    f.line([(745, top + 56), (745, lib - 20)], "model", arrow=False)
    f.line([(370, lib - 20), (915, lib - 20)], "model", arrow=False)
    names = [
        ("read back", "IR admission"),
        ("route", "candidates"),
        ("negotiate", "backends"),
        ("generate", "code · claims"),
    ]
    xs = [370, 530, 690, 850]
    for x, (name, sub) in zip(xs, names):
        f.line([(x + 65, lib - 20), (x + 65, lib - 3)], "model")
        f.box(x, lib, 130, 56, "logic", name, sub)
    for x in xs[:3]:
        f.line([(x + 130, lib + 28), (x + 160, lib + 28)], "logic")

    plan = 380
    f.text(0, plan - 12, "PLANNED", "muted", "mono", SMALL)
    f.box(780, plan, 200, 56, "logic", "quire prove · generate", "command verbs", dashed=True)
    f.line([(915, lib + 56), (915, plan - 3)], "logic", dashed=True)
    f.box(480, plan, 240, 56, "evidence", "verdict", "evidence from a backend", dashed=True)
    f.line([(780, plan + 28), (723, plan + 28)], "evidence", dashed=True)
    f.legend(0, 464, [("logic", "works today", "box"), ("logic", "planned", "dash")])
    return f


REPOS = [
    ("ORCHESTRATION", [("quire-verification", True), ("quire-driver", True), ("quire-integration", True)]),
    ("BACKENDS", [("quire-contract-codegen", False), ("quire-contract-runtime", False),
                  ("quire-analyze", False), ("quire-protocol", True), ("quire-mltl", False)]),
    ("LANGUAGE", [("quire-specification", True), ("quire-spec-language", False), ("quire-contract-ir", False)]),
    ("FOUNDATIONS", [("quire-exact", False), ("quire-semantic-value", False), ("quire-walk", False),
                     ("quire-canonical", False), ("quire-verification-contracts", False),
                     ("quire-observation", False)]),
]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f


def ecosystem_here():
    """The ecosystem figure with quire-driver highlighted."""
    return ecosystem(here="quire-driver")


FIGURES = [hero, ecosystem_here]
