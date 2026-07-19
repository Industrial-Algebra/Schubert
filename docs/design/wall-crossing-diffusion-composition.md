# Compositional Wall-Crossing for Diffusion-Language-Model Composition

> **Status:** Ideation / position document for the v0.5.0 research direction
> (Roadmap #17). Written to be examined during a long-term ideation session on
> Quantizon's roadmap extension. **This is a framing document, not a claim of
> solved results.** Established mathematics and existing Schubert code are stated
> as fact; the diffusion-LM mapping is an *analogy to be tested*, and is marked
> as such throughout.

## 1. The thesis

Quantizon's roadmap is extending in a specific direction:

> **From converting** open autoregressive LMs → diffusion
> **to composing** *new* diffusion LMs from open weights.

Conversion (AR → Diffusion → BlockDiffusion) is, mechanically, a per-model
transformation. **Composition** is a different and harder problem: given two or
more open-weight diffusion LMs, assemble a new one along some shared interface,
and reason about *what the assembled model can do*.

The hard part is not the mechanics of gluing weights — model merging,
mixture-of-experts routing, and weight interpolation all exist. The hard part is
**predicting the behavior of the composed model from its constituents**, and
especially **recognizing emergence**: behaviors the composition exhibits that
none of the constituents had. Weight averaging has no language for this. It
treats a model as a bag of numbers and hopes.

Schubert's **compositional wall-crossing** (Roadmap #17) is a candidate
mathematics for exactly this. It already models, in code, how a *capability set*
degrades across a continuous parameter (trust), where the degradation
"breakpoints" (walls) fall, and — centrally for this document — it poses the
open question of whether that structure **composes** under a gluing operation.
This document maps that structure onto diffusion-LM composition and lays out the
open questions worth examining.

The bonus: Quantizon already speaks **tropical algebra** (amari-tropical,
Bayesian masks), and tropical geometry is the native computational language of
wall-crossing (scattering diagrams). So the bridge does not require Quantizon to
import an alien formalism — it requires recognizing that two things it already
touches are the same thing.

## 2. What Schubert has today (established)

The wall-crossing machinery is not hypothetical — it ships in `schubert` and is
backed by `amari_enumerative`. From `src/stability.rs` and the controller:

- **`TrustLevel(t)`** ∈ [0, 1] — a continuous scalar parameter. `FULL` (1.0) =
  everything stable; `NONE` (0.0) = nothing.
- **Capability `σ_λ`** — a Schubert class with a dimension and codimension.
- **Phase of a capability** at trust `t`:

  ```
  phase(σ_λ, t) = (1/π) · arctan( t · dim(σ_λ) / codim(σ_λ) ) + 1/2
  ```

  Higher-codimension capabilities (e.g. an Admin-grant) become unstable at a
  *higher* trust level than lower-codimension ones (e.g. a Read-grant). Trust
  degrades capabilities in codimension order.
- **Wall** — the trust level at which a specific capability crosses its
  stability threshold (stable ↔ unstable).
- **Phase diagram `P`** — the piecewise-constant function `t ↦ #{stable
  capabilities at t}`. Its breakpoints are the walls. Computed today by
  `amari_enumerative::WallCrossingEngine::phase_diagram`.
- **Operadic composition `A ∘_S B`** — two principals (capability-bearing objects)
  composed along a shared capability set `S`. *Implemented for capability
  intersection; the wall-crossing behavior of the composition is the open
  question.*

### The #17 question

> If `C = A ∘_S B`, is the composed phase diagram `P_C` determined by `P_A` and
> `P_B`? I.e. is there a **Kontsevich–Soibelman (KS)-type formula** such that
> `P_C = F(P_A, P_B, S)`?

In physics, BPS bound states cross walls differently than their constituents —
the spectrum of a bound state is *not* the union of the constituent spectra.
That non-additivity is precisely **emergence**, formalized. If Schubert
satisfies a KS-type relation, then:

- where the relation holds, composition is *predictable* (the composed diagram
  is computable from the parts);
- where it *fails*, composition is *emergent* (the bound system has stable
  states no constituent had) — and the failure itself is the mathematically
  precise signature of emergence.

This is the document's central lever, and it lands directly on a concept
Quantizon already names: its **`Emergent` claim tier** ("possible and
intentionally left emergent", `docs/infrastructure-paper.md`, `quantizon-verify`).

## 3. The mapping (analogy — to be tested, not assumed)

| Schubert wall-crossing (established) | Diffusion-LM composition (proposed) |
|---|---|
| Principal `A` — a capability-bearing object on `Gr(k,n)` | A diffusion LM (or an open-weight component) `A` |
| Capability `σ_λ` — a Schubert class | A **generation mode** of the model: a skill, modality, denoising regime, or stylic affordance |
| Trust level `t` ∈ [0,1] | A **composition/conditioning parameter**: interpolation weight, gate temperature, constraint strength, denoising-step budget |
| `phase(σ_λ, t)` — is the mode stable at `t`? | Is generation mode `σ_λ` *active/realizable* under parameter `t`? |
| **Wall** — `t` where a mode appears/disappears | A composition-parameter value at which a generation mode switches on/off |
| **Phase diagram** `P` — `t ↦ #{active modes}` | The model's **behavior spectrum**: which modes are realizable as the parameter varies |
| `A ∘_S B` — compose along shared capability set `S` | Compose diffusion LMs `A`, `B` along a **shared interface** `S` (shared weight subspace, shared modality, shared context/token region) |
| KS-type formula `P_C = F(P_A, P_B, S)`? | Does the composed model's behavior spectrum derive from the constituents? |
| **BPS bound states** (non-additive spectrum) | **Emergent generation modes** present in `C` but in neither `A` nor `B` |

The right-hand column is a *hypothesis about a correspondence*, not a theorem.
Each row is a load-bearing identification that the ideation session should
stress-test (§6). The rows most likely to break: "what exactly is `t`?" and
"what exactly is the shared interface `S` for open weights?"

## 4. Why this is worth examining (not why it's correct)

1. **It gives composition a notion of *behavior*, not just weights.** A composed
   model is not defined by its parameter tensor; it is defined by *what it can
   generate*. The phase diagram is a behavior-level descriptor. Two merges with
   identical weight-norm statistics can have different phase diagrams — and the
   diagrams would say so.
2. **It separates predictable from emergent composition, formally.** Today,
   "emergence" in model composition is folk terminology. The KS-failure locus is
   a *defined, testable* boundary: composition is emergent exactly where
   `P_C ≠ F(P_A, P_B, S)`. This maps onto Quantizon's `Emergent`/`Rare`/
   `Impossible` claim tiering — giving that tiering a mathematical referent.
3. **It reuses Quantizon's existing tropical infrastructure.** Wall-crossing is
   computed via **scattering diagrams**, which are tropical objects. Quantizon
   already links amari-tropical. The bridge is "your tropical masks and Schubert's
   phase diagrams are two views of the same scattering diagram." (§5.)
4. **It composes recursively.** Operadic gluing is inherently n-ary and
   associative-up-to-coherence. "Compose new diffusion LMs from open weights"
   is naturally a many-part, recursive construction — operads are the standard
   language for exactly that.

## 5. The tropical bridge (where Quantizon has traction)

Wall-crossing, scattering diagrams, and tropical geometry are the same subject
viewed three ways:

- A **scattering diagram** is a piecewise-linear (tropical) object that records
  how rays (walls) mutate as you cross them. It is the data structure behind the
  KS formula.
- **Tropicalization** turns algebraic intersection problems (Schubert calculus)
  into piecewise-linear ones that are combinatorially computable.
- Schubert's `WallCrossingEngine` already lives in this world (via
  `amari_enumerative`), and Quantizon's masks already speak tropical (via
  `amari-tropical` / Bayesian masks).

So the proposed engine for "compute the composed phase diagram" is a scattering
diagram over the composition parameter space. The practical read for the
ideation session:

> *If diffusion-LM modes are tropical rays in a scattering diagram over the
> composition parameter, then wall-crossing is literally the mutation rule that
> says which modes exist in which chamber — and the composed diagram is computed
> by gluing scattering diagrams along `S`.*

Whether diffusion-LM modes actually *are* well-described as tropical rays is an
empirical question (§6). But the *machinery to test it* is already in the IA
stack.

## 6. Open questions for the ideation session

These are the load-bearing unknowns. They are questions, deliberately.

1. **What is `t`, concretely?** In Schubert, `t` is "trust" — a single scalar.
   In diffusion composition, is the natural parameter (a) a single interpolation
   weight, (b) a vector of per-component gate temperatures, or (c) the denoising
   step / noise schedule itself? The choice changes everything downstream. *My
   current lean: start with (a) as the simplest falsifiable case; generalize
   only if it bears fruit.*
2. **What is a "mode" `σ_λ`, operationally?** A mode must be something
   *countable* and *binary-at-a-wall*. Candidate operationalizations: a
   detectable stylistic/skill cluster in generation outputs; a stable attractor
   of the denoising dynamics; a capability measurable by a probe. Without a
   falsifiable definition of "mode," the phase diagram is not empirically
   computable.
3. **What is the shared interface `S` for open weights?** Candidates: a shared
   embedding/token space; a shared subset of layers held frozen during gluing; a
   shared modality (e.g., both constituents handle text). `S` is the operadic
   boundary — getting it wrong means the gluing isn't even well-typed.
4. **Is there a KS-type relation, and where does it break?** The single most
   important experiment: for a concrete `A`, `B`, `S`, measure `P_A`, `P_B`,
   `P_C` empirically and ask whether `P_C` is a function of the first two. The
   *loci where it isn't* are the emergence signatures.
5. **Relation to Quantizon's existing merge semantics.** Quantizon already has a
   merge policy (`docs/backend-output-contract.md` "Merge semantics"). Is that
   merge a *special case* of operadic gluing (single-component, no shared `S`
   beyond full overlap)? If so, the existing merge is the `S = everything`
   degenerate case, and operadic gluing generalizes it.
6. **Does the flag-variety embedding (#18) matter here?** Composing diffusion
   LMs that live in different "spaces" (different tokenizers, different
   modalities) may require embedding both into a common flag variety before
   gluing. #18 (flag variety embedding for cross-Grassmannian translation) may be
   a prerequisite, not a sibling.
7. **What is empirically testable on Quantizon's current hardware/crates?**
   Compositional wall-crossing is "research-grade" (#17 scope). The ideation
   should identify the smallest experiment that could *falsify* the framing,
   runnable with what Quantizon has (candle, amari-tropical, the AR→Diffusion
   artifacts already produced).

## 7. Candidate falsifiable spikes (small, killable)

Ordered cheapest-first. Each is designed to *falsify* a piece of the framing
cheaply, not to confirm it.

- **S0 — Modes-as-breakpoints probe.** Take one diffusion LM. Vary a single
  composition parameter (e.g., guidance scale). Detect "modes" via output
  clustering. *Question: does the mode-count look piecewise-constant with sharp
  breakpoints (a phase diagram), or smooth?* If smooth, the wall-crossing
  framing may not apply and should be dropped early.
- **S1 — Does composition break additivity?** Compose `A`, `B` trivially
  (full-overlap merge, `S = everything`). Measure `P_A`, `P_B`, `P_C`. *Question:
  is `P_C` close to a function of `P_A`, `P_B`?* Establishes a baseline before
  introducing real shared-interface gluing.
- **S2 — Emergence-at-walls.** In a composition where emergence is suspected,
  *do the emergent modes first appear at composition-parameter values that are
  walls in the constituent diagrams?* This is the most direct test of the
  BPS/ emergence identification. If emergent modes appear smoothly away from any
  wall, the BPS analogy is wrong.
- **S3 — Tropical ray encoding.** Attempt to encode a diffusion LM's behavior
  spectrum as a tropical scattering diagram using amari-tropical, and check that
  gluing two diagrams reproduces S1's measured `P_C`. The real test of the
  "machinery is already in the stack" claim.

## 8. Relationship to Schubert's roadmap and publication

- This is the **application grounding for Roadmap #17** (compositional
  wall-crossing). The Schubert-side implementation —
  `analyze_composed_stability()` — tests the KS-type relation on principals;
  this document argues the *same question* is the right frame for Quantizon's
  composition roadmap.
- **Roadmap #18** (flag variety embedding) may be a prerequisite for
  cross-space composition (§6.6), not merely a sibling.
- **Publication:** this maps to **Ch. 8 of the revised arXiv preprint**
  (composition and the open question). A diffusion-LM application would convert
  #17 from an internal-theoretical result into a *motivated external
  application* — substantially strengthening the paper.

## 9. What this document is not

- **Not a theorem.** No claim that `P_C = F(P_A, P_B, S)` holds for diffusion
  LMs. The mapping in §3 is an analogy; §6 lists exactly where it may break.
- **Not a spec.** No API, no crate layout, no implementation plan. Those follow
  *after* the ideation session decides which (if any) sub-direction is worth
  building.
- **Not a commitment to implement #17 in v0.5.0.** #17 is explicitly
  "research-grade." The realistic v0.5.0 outcome may be "framing validated (or
  falsified) + one spike," with the math proper deferred.

## 10. Decision asks for the ideation session

1. Does the §3 mapping survive scrutiny, or does a row collapse (killing the
   framing)?
2. Which `t` (§6.1) and which `S` (§6.3) do we commit to for the first
   experiment?
3. Is S0 cheap enough to run *before* committing further — i.e., do we falsify
   first?
4. Does this belong in Schubert (as #17 application), in Quantizon (as roadmap
   math), or as a cross-project IA research note?

---

*Cross-references: Schubert `docs/ROADMAP.md` §17–18; `src/stability.rs`;
`amari_enumerative::WallCrossingEngine`. Quantizon `docs/architecture.md`,
`docs/infrastructure-paper.md` (`Emergent` claim tier),
`docs/backend-output-contract.md` (merge semantics), `amari-tropical` masks.*
