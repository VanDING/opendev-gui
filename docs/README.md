# OpenDev Desktop — Governance & Documentation

This directory is the authoritative source for project governance, architecture decisions,
engineering standards, and contributor guidance.

## Directory Map

| Directory | Purpose | Stability |
|---|---|---|
| `adr/` | Architecture Decision Records | **Frozen** — each ADR is immutable once accepted |
| `architecture/` | Current architecture documentation | Stable — updates when architecture changes |
| `engineering/` | Engineering standards and practices | Stable — evolves with team consensus |
| `contributors/` | Onboarding and contribution guides | Active — updated as project evolves |
| `constitution.md` | Cabinet design principles (10-15 rules) | **Frozen** — changes only by project-wide consensus |
| `roadmap.md` | Agreed next steps for the coding agent | Active — updated per release cycle |

## Principles

1. **Governance documents describe current truth, not future vision.** Every document here
   records what has been decided, built, or agreed — not what is planned.
2. **ADR entries are immutable.** Once accepted, an ADR records a decision that was made.
   Superseding it requires a new ADR that references the old one.
3. **Root-level documents (`ARCHITECTURE.md`, `DESIGN.md`) remain at root** for discoverability.
   The `docs/` directory adds depth for governance, ADRs, and contributor guides.

## Relationship to Root Files

| Root File | Purpose | Relation to `docs/` |
|---|---|---|
| `ARCHITECTURE.md` | High-level architecture overview | Referenced by `architecture/` sub-docs |
| `DESIGN.md` | Visual design specification | Referenced by engineering standards |
| `CHANGELOG.md` | Release history | Referenced by roadmap |
| `README.md` | Project quick-start | Referenced by contributor guides |

## Who Maintains What

| Section | Maintainer | Review Cadence |
|---|---|---|
| `constitution.md` | Project lead | Only on principle changes |
| `adr/` | Decision author + project lead | Per ADR submission |
| `architecture/` | Architecture team | Per crate restructuring |
| `engineering/` | Engineering team | Per tooling/practice change |
| `contributors/` | Any contributor | Continuous — update when onboarding reveals gaps |
| `roadmap.md` | Project lead | Per release cycle |
