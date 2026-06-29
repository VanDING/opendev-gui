---
name: ontological-research
description: Domain-independent ontological research through systematic elimination, cross-disciplinary comparison, and falsification-first stress-testing. Use ONLY when the user explicitly asks to conduct ontological research on a concept or phenomenon — establishing what it fundamentally IS before any product design, engineering, or implementation discussion.
---

# Ontological Research Protocol

## When to Use

Invoke this skill when the user asks to research a phenomenon's ontology — "what is X, fundamentally, independent of any particular implementation, methodology, or technology?"

Typical triggers: "research the ontology of...", "what is... fundamentally?", "establish a theoretical foundation for..."

Do NOT use for: product design, feature analysis, engineering discussions, literature reviews, or any task where "what exists" is already assumed.

---

## Core Principles

### Principle 1 — Ontology Before Everything
Always establish what something IS before asking how it works, how it's managed, or how it's represented. Chapter 1 always precedes Chapters 2–6.

### Principle 2 — Eliminate, Do Not Collect
Never accumulate definitions. Ruthlessly eliminate candidates. "Can the phenomenon still exist without this?" If yes → remove. 30 candidates → 4 primitives is success. 30 candidates → 30 features is failure.

### Principle 3 — Cross-Disciplinary Invariants
Test every candidate across at minimum 8 disciplines (PMBOK, systems theory, cybernetics, software engineering, architecture, scientific research, manufacturing, organizational theory). If a candidate doesn't appear in ALL disciplines, it's not an invariant.

### Principle 4 — Falsification Over Confirmation
Always attempt to break the theory. The best finding is "this candidate does NOT survive." Confirmation is worthless; falsification reveals the true boundary.

### Principle 5 — Theory vs. Implementation
Strictly separate: Ontology (what exists) → Dynamics (how it changes) → Governance (how it's regulated) → Representation (how it's projected) → Implementation (how it's realized). Never mix layers. Lower layers cannot modify higher layers.

### Principle 6 — Products Are Evidence, Never Theory
Products may falsify or strengthen theory. They may never introduce new primitives. The Iron Law: any ontology revision from product analysis must survive independent theoretical validation first.

### Principle 7 — One Concept, One Definition, One Location
Every concept has exactly one canonical definition. No redefinition in later chapters. Later chapters reference earlier definitions.

---

## Research Framework

### Nine-Dimensional Analysis

Every chapter in the research follows this unified framework:

| # | Dimension | Core Question |
|---|-----------|---------------|
| 1 | **Definition** | What is it? |
| 2 | **Necessity** | Why does it exist? |
| 3 | **Essential Properties** | What attributes, if removed, make it cease to be itself? |
| 4 | **Evolution** | Why did it evolve into its current form? |
| 5 | **Existing Models** | How do different disciplines explain it? |
| 6 | **Practical Manifestation** | What real-world examples exist? |
| 7 | **Contradictions** | What phenomena can current theories not explain? |
| 8 | **AI Era Changes** | What fundamental assumptions change? |
| 9 | **Implications** | What does this mean for design/implementation? |

### Six-Chapter Architecture

If the research spans a full theoretical system, organize as:

```
Ch1: Ontology       — defines    (what X fundamentally is)
Ch2: Evolution      — changes    (how X changes over time)
Ch3: Governance     — regulates  (how X's change is governed)
Ch4: Representation — projects   (how X is represented in tools/systems)
Ch5: Stress-Test    — validates  (test against the strongest counterexample available)
Ch6: Implementation — derives    (what must an environment provide to support X)
```

Each chapter is downstream from the previous. Each chapter's conclusions derive from earlier chapters.

---

## Core Protocols

### Protocol 1 — Elimination Methodology

For any candidate element/primitive/property:

1. **Necessity Test.** Can the phenomenon exist without this candidate? If yes → REMOVE.
2. **Sufficiency Test.** Does this candidate alone make something an instance of the phenomenon? If no → it's not sufficient; keep only if necessary.
3. **Independence Test.** Can this candidate be derived from other candidates? If yes → REMOVE (merge into the candidate it derives from).
4. **Cross-Disciplinary Test.** Does this candidate appear across ALL 8+ disciplines? If no → NOT an invariant.
5. **Human Replacement Test.** If you replace every instance of the candidate with a human equivalent, does the proposition still hold? If yes → the candidate is implementation, not ontology.

Target: compress N candidates to the smallest irreducible set (typically 3-7).

### Protocol 2 — Human Replacement Test (Ch5-specific)

For any claim about how a new phenomenon changes things:

> "If I replace [new phenomenon] with a sufficiently capable human, does this proposition still hold?"

- YES → implementation change (speed, scale, cost), not ontological change
- NO → genuinely new ontological category

This test prevents conflating "capability amplification" with "ontological revolution."

### Protocol 3 — Audit Protocol

After each research part, conduct a formal audit:

1. **Logical Audit.** Circular definitions? Hidden assumptions? Undefined terms? Invalid deductions?
2. **Internal Consistency.** Contradictions between chapters? Concept drift?
3. **Boundary Testing.** Edge cases. Can the theory explain them or legitimately exclude them?
4. **Reduction Test.** Can anything be further compressed? Can two concepts merge?
5. **Falsification Attempt.** Search for counterexamples. Try to break the theory.

The auditor role is skeptic, not collaborator. "Not finding counterexamples" is insufficient; survival must be grounded in demonstrated logical necessity.

### Protocol 4 — Layer Separation

Every statement must belong to exactly one layer:

| Layer | Question | Allowed | Forbidden |
|-------|----------|---------|-----------|
| **Theory** | What must exist? | Definitions, axioms, theorems, necessary conditions | Products, AI, engineering, cases, APIs |
| **Specification** | What must implementation satisfy? | MUST/SHOULD/MAY, interfaces, schemas, compliance rules | Proofs, philosophy, product discussion |
| **Implementation** | How is it realized? | Architecture, patterns, data models, engineering choices | Modifying specification, changing theory |
| **Explanation** | How do humans understand it? | Books, papers, cases, courses, narratives | Modifying any layer above |

Dependency is strictly downward. Implementation cannot change Specification. Specification cannot change Theory.

### Protocol 5 — Representation Iron Law

When analyzing products/tools/systems:

> Products may falsify or strengthen theory through empirical stress-testing. They may NOT introduce new ontological primitives. If a product appears to reveal a new dimension, that dimension must survive independent theoretical validation before incorporation.

Direction of reasoning: Theory → Prediction → Product analysis confirms or challenges. NOT: Product analysis → Pattern extraction → Theory.

---

## Writing Discipline

### Canonical Specification Style
- Delete ALL research process (discussions, audits, "we found...", "later revised...")
- One concept, one definition, one location
- Number all propositions (Definition 1, Theorem 2, Necessary Condition 3)
- MUST/SHOULD/MAY for specification language; never for theory
- All open questions → Appendix
- All examples → Appendix
- All products → Appendix
- Delete all rhetoric ("interestingly...", "profound...", "surprisingly...")
- Every sentence must justify its existence: if removed, does the theory change? If no → remove.

### Chapter Dependencies
Every chapter ends with explicit dependency declaration:
```
Requires: Definition 1–8, Theorem 1–3
Provides: Definition 9–14, Theorem 4–7
```

The full system must form a DAG. No circular dependencies.

---

## Quality Gates

### Per-Part Gate
- [ ] 9-dimension framework applied?
- [ ] Cross-disciplinary (8+ disciplines)?
- [ ] Elimination over collection?
- [ ] Human Replacement Test applied (if relevant)?
- [ ] Audit completed?

### Per-Chapter Gate
- [ ] Definitions are unique and non-overlapping?
- [ ] No ontological drift from previous chapters?
- [ ] Dependency DAG maintained?
- [ ] Boundary conditions explicit?
- [ ] Open questions documented?

### Final Gate (v1.0)
- [ ] Canonical Specification produced?
- [ ] Independent validation passed?
- [ ] Layer separation audit passed?
- [ ] Acceptance test (10 criteria) passed?

---

## Delegation Patterns

### When to use @librarian
- Cross-disciplinary research requiring current library docs, API references, examples
- "How is this concept defined across PMBOK, systems theory, cybernetics...?"
- External literature review with specific sources needed

### When to use @oracle
- Architecture decisions with long-term impact
- Problems persisting after 2+ fix attempts
- High-risk multi-system refactors
- Complex debugging with unclear root cause
- Code review, simplification, maintainability review

### When the Orchestrator manages directly
- Internal consistency audits
- Synthesis across completed research parts
- Protocol execution (elimination tests, layer separation)
- Canonical Specification production
- Research that requires full context of all previous work

---

## Version History

| Version | Source | Key Features |
|---------|--------|--------------|
| v1.0 | Project Theory Ch1–6 complete research cycle | Full 9-dimension framework, 6-protocol suite, elimination methodology, Human Replacement Test, Iron Law, layer separation |
