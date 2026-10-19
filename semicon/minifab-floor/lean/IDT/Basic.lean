/-
W668 IDT mechanization wave — shared scaffold (M1 owns this file; FROZEN interface).
-/
import BernsteinLogicAux.Imprecise.Credal

/-! # IDT — Imprecise Decision Theory (MWER + EDT + FNC): the shared core

**Theory name** per davidad's 2026-07-31 christening (W668 log): **IDT** —
minimax weighted expected regret (Halpern–Leung) over a belief stock, with the
two free parameters filled by two Observed-predicates: FNC (the existential
anthropic update) and EDT (the `∀m` policy-conditioning event).

**Source specs**: `hearth/maitria/w666-receipts/W668-THEOREMS.md` §0 (the
finite program: stock, likelihood updating, MWER, fiber menus, locality);
`hearth/maitria/w666-receipts/W668-MENU-BATTERY.md` §0 (the scenario
template); HL = arXiv:1302.5681 (MWR.tex, regret + weighted-regret displays,
likelihood updating at 1036–1076).

**Ownership (reverse-Conway, W668 wave)**: M1 freezes this file; siblings
import it and own disjoint files — M2 `IDT/Composition.lean` +
`IDT/Selection.lean`, M3 `IDT/Representation.lean`, M4 `IDT/SureLoss.lean` +
`IDT/Shield.lean`; M1's own scope is `IDT/Battery.lean` +
`IDT/ErasureWitness.lean`.

**Design choices, load-bearing:**

- Everything is exact-`ℚ` over `Fintype`s, riding
  `BernsteinLogic.Imprecise.FinPMF` (the W408 credal core) — build beside the
  existing kernel-checked stack, never re-derive it.
- Menus are **indexed families** `ι → X → ℚ` (an act = a map from worlds to
  `ℚ`-utilities; a policy enters as its act `x ↦ u(x, π)`), matching
  `Credal.lean`'s generating-family style and keeping every operator
  computable.
- **Stocks carry unnormalized weights** (only `0 ≤ wt`). HL's sup-
  normalization (`max_k α_k = 1`) is a *positive rescaling* of the weighted-
  regret functional, so it never moves an argmin; it is available as the
  `Stock.Normalized` predicate for the representation lane (M3), where the
  canonical-body uniqueness clause genuinely needs it.
- **Likelihood updating is total and ungrouped**: measures with
  `mass E = 0` are left unchanged and silenced by their zero updated weight
  (they contribute `0` to a max of nonnegatives — `regret_nonneg`); HL's
  grouping of equal conditionals changes no `wregret` value (the group max
  and the pointwise max coincide on regret terms that depend on `k` only
  through the conditional) — M3 owns that lemma; the preference functional
  here is already HL's.
- **`by decide` cannot evaluate `ℚ` at this toolchain/mathlib pin**
  (banked in `TowerWitness.lean`): concrete cells are crunched by
  `simp`/`norm_num` through the `le_antisymm` API below (`wregret_le` +
  `le_wregret`, `best_le` + `exp_le_best`), never by kernel-whnf of `Rat`.
-/

namespace BernsteinLogic.IDT

open BernsteinLogic.Imprecise
open Finset

/-! ## Small `FinPMF` constructors (shared by every battery row) -/

namespace FinPMFExtra

variable {A : Type*} [Fintype A]

/-- Dirac mass at `a`. -/
def pure [DecidableEq A] (a : A) : FinPMF A where
  prob := fun b => if b = a then 1 else 0
  nonneg := fun b => by by_cases h : b = a <;> simp [h]
  total := by simp

@[simp] theorem pure_prob [DecidableEq A] (a b : A) :
    (pure a).prob b = if b = a then 1 else 0 := rfl

/-- Bernoulli pmf on `Bool`: mass `p` on `true`, `1 - p` on `false`. -/
def bernoulli (p : ℚ) (h0 : 0 ≤ p) (h1 : p ≤ 1) : FinPMF Bool where
  prob := fun b => cond b p (1 - p)
  nonneg := fun b => by cases b <;> simpa using by linarith
  total := by simp [Fintype.univ_bool]

@[simp] theorem bernoulli_prob_true (p : ℚ) (h0 : 0 ≤ p) (h1 : p ≤ 1) :
    (bernoulli p h0 h1).prob true = p := rfl

@[simp] theorem bernoulli_prob_false (p : ℚ) (h0 : 0 ≤ p) (h1 : p ≤ 1) :
    (bernoulli p h0 h1).prob false = 1 - p := rfl

end FinPMFExtra

/-! ## Events: mass and conditioning (the FNC step's engine)

Events are `Finset X`. `FinPMF.cond` is TOTAL: on a zero-mass event it
returns the pmf unchanged — the caller (`Stock.update`) zeroes the weight, so
the junk conditional is preference-inert. -/

namespace Event

variable {X : Type*} [Fintype X]

/-- Probability mass of the event `E`. -/
def mass (p : FinPMF X) (E : Finset X) : ℚ := ∑ x ∈ E, p.prob x

theorem mass_nonneg (p : FinPMF X) (E : Finset X) : 0 ≤ mass p E :=
  Finset.sum_nonneg fun x _ => p.nonneg x

theorem mass_le_one (p : FinPMF X) (E : Finset X) : mass p E ≤ 1 := by
  have h := Finset.sum_le_sum_of_subset_of_nonneg (Finset.subset_univ E)
    (fun x _ _ => p.nonneg x)
  simpa [mass, p.total] using h

/-- Conditioning on `E` (total: identity on zero-mass events). -/
def condPMF [DecidableEq X] (p : FinPMF X) (E : Finset X) : FinPMF X :=
  if h : 0 < mass p E then
    { prob := fun x => if x ∈ E then p.prob x / mass p E else 0
      nonneg := fun x => by
        by_cases hx : x ∈ E
        · simp only [hx, if_true]
          exact div_nonneg (p.nonneg x) (le_of_lt h)
        · simp [hx]
      total := by
        rw [Finset.sum_ite_mem, Finset.univ_inter]
        simp only [div_eq_mul_inv]
        rw [← Finset.sum_mul]
        exact mul_inv_cancel₀ (ne_of_gt h) }
  else p

theorem condPMF_of_pos [DecidableEq X] {p : FinPMF X} {E : Finset X}
    (h : 0 < mass p E) :
    (condPMF p E).prob = fun x => if x ∈ E then p.prob x / mass p E else 0 := by
  simp [condPMF, h]

theorem condPMF_of_zero [DecidableEq X] {p : FinPMF X} {E : Finset X}
    (h : ¬ 0 < mass p E) :
    condPMF p E = p := by simp [condPMF, h]

end Event

/-! ## The belief stock and its likelihood update -/

/-- A (finitely generated) **belief stock** `𝒫⁺ = {(Pr_k, α_k)}`: weighted
priors, weights nonnegative, indexed by a (typically `Fintype`) index `K`.
Sup-normalization is NOT a field (see module docstring); `Stock.Normalized`
carries it where needed. -/
structure Stock (X : Type*) [Fintype X] (K : Type*) where
  /-- The measures. -/
  pr : K → FinPMF X
  /-- The weights. -/
  wt : K → ℚ
  /-- Weights are nonnegative. -/
  wt_nonneg : ∀ k, 0 ≤ wt k

namespace Stock

variable {X : Type*} [Fintype X] {K : Type*}

/-- HL's sup-normalization: weights in `[0,1]` attaining `1`. -/
def Normalized [Fintype K] (S : Stock X K) : Prop :=
  (∀ k, S.wt k ≤ 1) ∧ ∃ k, S.wt k = 1

/-- The singleton (precise) stock. -/
def precise (p : FinPMF X) : Stock X Unit where
  pr := fun _ => p
  wt := fun _ => 1
  wt_nonneg := fun _ => zero_le_one

@[simp] theorem precise_pr (p : FinPMF X) (k : Unit) : (precise p).pr k = p := rfl
@[simp] theorem precise_wt (p : FinPMF X) (k : Unit) : (precise p).wt k = 1 := rfl

/-- **Likelihood updating** on event `E` (HL, MWR.tex:1036–1076, ungrouped
and unnormalized — see module docstring): each measure conditions on `E`,
each weight rescales by `Pr_k(E)`. Zero-mass measures keep their pmf
(`condPMF` is total) and get weight `0`. -/
def update [DecidableEq X] (S : Stock X K) (E : Finset X) : Stock X K where
  pr := fun k => Event.condPMF (S.pr k) E
  wt := fun k => S.wt k * Event.mass (S.pr k) E
  wt_nonneg := fun k => mul_nonneg (S.wt_nonneg k) (Event.mass_nonneg _ _)

@[simp] theorem update_pr [DecidableEq X] (S : Stock X K) (E : Finset X) (k : K) :
    (S.update E).pr k = Event.condPMF (S.pr k) E := rfl

@[simp] theorem update_wt [DecidableEq X] (S : Stock X K) (E : Finset X) (k : K) :
    (S.update E).wt k = S.wt k * Event.mass (S.pr k) E := rfl

end Stock

/-! ## Menus, regret, MWER

A **menu** is an indexed family of acts `M : ι → X → ℚ` over a nonempty
`Fintype` index. `best M p` is the menu benchmark, `regret M p i` the
per-measure regret, `wregret S M i` the weighted regret, and
`IsMWERChoice S M i` the acceptance/selection predicate (HL: choose the
argmin of max weighted regret). -/

section MWER

variable {X : Type*} [Fintype X] {K : Type*} [Fintype K] [Nonempty K]
variable {ι : Type*} [Fintype ι] [Nonempty ι]

/-- Menu benchmark: the best expected utility available in `M` under `p`. -/
def best (M : ι → X → ℚ) (p : FinPMF X) : ℚ :=
  Finset.univ.sup' Finset.univ_nonempty fun j => p.exp (M j)

/-- Regret of act `i` against menu `M` under measure `p`. -/
def regret (M : ι → X → ℚ) (p : FinPMF X) (i : ι) : ℚ :=
  best M p - p.exp (M i)

/-- Weighted regret of act `i` against menu `M` under stock `S`:
`max_k α_k · reg_M(i, Pr_k)`. -/
def wregret (S : Stock X K) (M : ι → X → ℚ) (i : ι) : ℚ :=
  Finset.univ.sup' Finset.univ_nonempty fun k => S.wt k * regret M (S.pr k) i

/-- MWER acceptance/selection: `i` minimizes max weighted regret over the
menu. (HL's chosen set; the battery's cells each pin one of these.) -/
def IsMWERChoice (S : Stock X K) (M : ι → X → ℚ) (i : ι) : Prop :=
  ∀ j, wregret S M i ≤ wregret S M j

/-! ### The `le_antisymm` API (the `norm_num` crunching route) -/

theorem exp_le_best (M : ι → X → ℚ) (p : FinPMF X) (j : ι) :
    p.exp (M j) ≤ best M p :=
  Finset.le_sup' (fun j => p.exp (M j)) (Finset.mem_univ j)

theorem best_le {M : ι → X → ℚ} {p : FinPMF X} {a : ℚ}
    (h : ∀ j, p.exp (M j) ≤ a) : best M p ≤ a :=
  Finset.sup'_le _ _ fun j _ => h j

theorem best_eq_of (M : ι → X → ℚ) (p : FinPMF X) (j : ι)
    (h : ∀ j', p.exp (M j') ≤ p.exp (M j)) : best M p = p.exp (M j) :=
  le_antisymm (best_le h) (exp_le_best M p j)

theorem regret_nonneg (M : ι → X → ℚ) (p : FinPMF X) (i : ι) :
    0 ≤ regret M p i :=
  sub_nonneg.mpr (exp_le_best M p i)

theorem le_wregret (S : Stock X K) (M : ι → X → ℚ) (i : ι) (k : K) :
    S.wt k * regret M (S.pr k) i ≤ wregret S M i :=
  Finset.le_sup' (fun k => S.wt k * regret M (S.pr k) i) (Finset.mem_univ k)

theorem wregret_le {S : Stock X K} {M : ι → X → ℚ} {i : ι} {a : ℚ}
    (h : ∀ k, S.wt k * regret M (S.pr k) i ≤ a) : wregret S M i ≤ a :=
  Finset.sup'_le _ _ fun k _ => h k

theorem wregret_eq_of (S : Stock X K) (M : ι → X → ℚ) (i : ι) (k : K)
    (h : ∀ k', S.wt k' * regret M (S.pr k') i ≤ S.wt k * regret M (S.pr k) i) :
    wregret S M i = S.wt k * regret M (S.pr k) i :=
  le_antisymm (wregret_le h) (le_wregret S M i k)

theorem wregret_nonneg (S : Stock X K) (M : ι → X → ℚ) (i : ι) :
    0 ≤ wregret S M i := by
  obtain ⟨k⟩ := ‹Nonempty K›
  exact le_trans (mul_nonneg (S.wt_nonneg k) (regret_nonneg M (S.pr k) i))
    (le_wregret S M i k)

/-- MWER choices exist on finite nonempty menus. -/
theorem exists_mwerChoice (S : Stock X K) (M : ι → X → ℚ) :
    ∃ i, IsMWERChoice S M i := by
  obtain ⟨i, -, hi⟩ := Finset.exists_min_image Finset.univ
    (fun i => wregret S M i) ⟨Classical.arbitrary ι, Finset.mem_univ _⟩
  exact ⟨i, fun j => hi j (Finset.mem_univ j)⟩

/-- On a precise (singleton) stock, weighted regret IS regret — the S1
precise-collapse engine (MWER = EU-maximization follows since `best` is
constant across candidates; M2 owns the preference-level statement). -/
theorem wregret_precise (p : FinPMF X) (M : ι → X → ℚ) (i : ι) :
    wregret (Stock.precise p) M i = regret M p i :=
  le_antisymm (wregret_le fun _ => by simp [Stock.precise])
    (by simpa [Stock.precise] using le_wregret (Stock.precise p) M i ())

/-! ### Congruence helpers (additive extension, M1 post-freeze — no
existing name changed) -/

theorem best_congr {M : ι → X → ℚ} {p q : FinPMF X}
    (h : ∀ j, p.exp (M j) = q.exp (M j)) : best M p = best M q := by
  apply le_antisymm
  · exact best_le fun j => le_trans (le_of_eq (h j)) (exp_le_best M q j)
  · exact best_le fun j => le_trans (le_of_eq (h j).symm) (exp_le_best M p j)

theorem regret_congr {M : ι → X → ℚ} {p q : FinPMF X}
    (h : ∀ j, p.exp (M j) = q.exp (M j)) (i : ι) :
    regret M p i = regret M q i := by
  simp only [regret, best_congr h, h i]

end MWER

/-- Two pmfs with equal mass functions have equal expectations — the
workhorse for mass-one conditioning (`condPMF p E` agrees with `p`
pointwise when the off-event mass is zero). -/
theorem exp_congr_prob {X : Type*} [Fintype X] {p q : FinPMF X}
    (h : p.prob = q.prob) (f : X → ℚ) : p.exp f = q.exp f := by
  simp only [FinPMF.exp, h]

/-! ## Acceptance-vector menus (M4's sure-loss lane)

A package of `n` offers evaluated as ONE menu of acceptance vectors — the
policy-level stance under which the R6(iii) myopic counterexample dissolves
(`W668-R6III-SURELOSS.md`): the acts are the `2^n` accept/reject vectors, a
dominated package is a dominated act. -/

/-- The act of acceptance vector `b` over `offers`, on top of `base`. -/
def acceptanceAct {X : Type*} {n : ℕ} (base : X → ℚ) (offers : Fin n → X → ℚ)
    (b : Fin n → Bool) : X → ℚ :=
  fun x => base x + ∑ i, if b i then offers i x else 0

/-- The acceptance-vector menu: all `2^n` accept/reject combinations. -/
def acceptanceMenu {X : Type*} [Fintype X] {n : ℕ} (base : X → ℚ)
    (offers : Fin n → X → ℚ) : (Fin n → Bool) → X → ℚ :=
  acceptanceAct base offers

/-! ## Anthropic scenarios: observer moments, the FNC trigger, the `∀m` step

The battery §0 template `(X, {M(x)}, Inputs, Outputs, u, 𝒫⁺, Π)`, flattened:
a `Fintype` of moments with world- and input-projections. The **FNC
trigger** `E_o = {x | ∃ m, world m = x ∧ inp m = o}` is the existential
Observed-predicate; the **`∀ m` (EDT) step** is policy substitution at one
input, evaluated under the independent-draw convention (`uMix`) or
deterministically (`uDet`). `Local` is THEOREMS §0's locality hypothesis —
`payoff x` reads only the outputs of `x`'s own moments (prereg T1-M3: an
assumption, stated, never smuggled). -/

/-- An anthropic scenario: finite moments, their worlds and inputs, and a
world-level payoff reading a full output assignment. -/
structure Scenario (X Mo I A : Type*) where
  /-- Which world each moment inhabits. -/
  world : Mo → X
  /-- The input (observation) each moment receives. -/
  inp : Mo → I
  /-- World-level payoff of a joint output assignment. -/
  payoff : X → (Mo → A) → ℚ

namespace Scenario

variable {X Mo I A : Type*}

/-- Locality (THEOREMS §0): the payoff in world `x` depends only on outputs
at `x`'s own moments. -/
def Local (S : Scenario X Mo I A) : Prop :=
  ∀ x f g, (∀ m, S.world m = x → f m = g m) → S.payoff x f = S.payoff x g

/-- The FNC trigger `E_o`: worlds containing at least one `o`-input moment. -/
def trigger [Fintype Mo] [DecidableEq X] [DecidableEq I]
    (S : Scenario X Mo I A) (o : I) : Finset X :=
  (Finset.univ.filter fun m => S.inp m = o).image S.world

theorem mem_trigger [Fintype Mo] [DecidableEq X] [DecidableEq I]
    (S : Scenario X Mo I A) (o : I) (x : X) :
    x ∈ S.trigger o ↔ ∃ m, S.world m = x ∧ S.inp m = o := by
  simp only [trigger, Finset.mem_image, Finset.mem_filter, Finset.mem_univ,
    true_and]
  exact ⟨fun ⟨m, hm, hw⟩ => ⟨m, hw, hm⟩, fun ⟨m, hw, hm⟩ => ⟨m, hm, hw⟩⟩

/-- Deterministic policy evaluation: every moment outputs per the policy at
its own input (the `∀ m` predicate, deterministic case). -/
def uDet (S : Scenario X Mo I A) (π : I → A) (x : X) : ℚ :=
  S.payoff x fun m => π (S.inp m)

/-- Mixed policy evaluation, **independent-draw convention**: each moment
draws independently from the policy at its input (behavioral strategies —
the AMD row's convention; THEOREMS §0's locality holds under either
convention, and the battery rows say which they use). -/
def uMix [Fintype Mo] [DecidableEq Mo] [Fintype A] (S : Scenario X Mo I A)
    (π : I → FinPMF A) (x : X) : ℚ :=
  ∑ f : Mo → A, (∏ m, (π (S.inp m)).prob (f m)) * S.payoff x f

/-- O&C Definition-6 substitution `π_{o→a}`: the policy equal to `π` except
outputting `a` at input `o` — the `∀ m` conditioning event as an operation. -/
def subst [DecidableEq I] (π : I → A) (o : I) (a : A) : I → A :=
  fun i => if i = o then a else π i

/-- Mixed substitution `π_{o→α}`. -/
def substMix [DecidableEq I] [Fintype A] (π : I → FinPMF A) (o : I)
    (α : FinPMF A) : I → FinPMF A :=
  fun i => if i = o then α else π i

/-- The deterministic **fiber menu** `F_{o,π}` at observation `o`, incumbent
`π`: deviations at `o` only, indexed by the deviating output. -/
def fiberMenuDet [DecidableEq I] (S : Scenario X Mo I A) (π : I → A) (o : I) :
    A → X → ℚ :=
  fun a => S.uDet (subst π o a)

/-- The mixed fiber menu over an index family of deviation pmfs. -/
def fiberMenuMix [DecidableEq I] [Fintype Mo] [DecidableEq Mo] [Fintype A]
    (S : Scenario X Mo I A) (π : I → FinPMF A) (o : I) {ι : Type*}
    (devs : ι → FinPMF A) : ι → X → ℚ :=
  fun d => S.uMix (substMix π o (devs d))

end Scenario

end BernsteinLogic.IDT
