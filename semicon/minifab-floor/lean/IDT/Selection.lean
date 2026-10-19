/-
W668 M2 — IDT mechanization: the Psy-Kosh selection principle and the
imprecise-extension counterexample (mechanization debts §3(2) and §3(5)).

Source of truth: `hearth/maitria/w666-receipts/W668-THEOREMS.md` §2 —
Theorem 2 (menu-MWER selection = ex-ante optimality; selection refines
ratification at precise stocks), the parametric Psy-Kosh family (battery
row 11), the family-wide full-menu check, and the §2.6 exact-ℚ 2×2×2
witness in which the UNIQUE global menu-MWER optimum is fiber-unstable at
BOTH inputs (the preregistered sub-claim that came out FALSE — first-class
cargo), with the singleton-limit control (§3(5)'s mutant: stability
restores as the second weight vanishes) — upgraded by the `Threshold`
section (W668 α₂-sweep leg) from two spot-checks to the exact parametric
threshold: stability restores at t₀ = 1/9 precisely, sharply, and
NON-monotonically (prereg: W668-ALPHA2-SWEEP-PREREG.md).

Statement-fidelity notes (M2-FINDINGS.md carries the fuller ledger):

* Ratifiability (`Ratifiable`) is defined POSTERIOR-side, exactly as paper
  §2.1: at every observation with positive `P̄`, the incumbent's output is
  an MWER choice in the post-update fiber menu. `ratifiable_iff_exante`
  (Theorem 1's consequence — "updating provably innocent", Corollary 1.2 /
  §2.6's reading) converts it to the ex-ante form.
* Deviations are UNRESTRICTED (fiber menus indexed by all of `A`), and
  Theorem 2(ii) is stated for substitution-closed policy menus. This is the
  form every downstream consumer uses (the doc's own family theorem is
  "ratifiable w.r.t. unrestricted fiber deviations"; the §2.6 witness's
  menu is a full product). The doc's Π-restricted fiber menus (benchmark
  restricted to `{α : π_{o→α} ∈ Π}`) are NOT formalized — a named
  weakening; nothing in the paper's §2 consumes the restricted form beyond
  substitution-closed instances.
* Deterministic policies only; for the Psy-Kosh fibers the paper's own
  input-injectivity remark (§2.6's existence proposition) makes fiber EU
  affine in the mixed deviation, so deterministic-endpoint ratifiability
  coincides with mixed ratifiability there — paper-grade remark, not
  re-proved here.
* Psy-Kosh T-decider selection: the paper's family leaves the k-subset
  distribution unspecified (the canonical instance is complement-of-one at
  n = 10, k = 9); we instantiate CYCLIC INTERVALS `S_j = {j, …, j+k−1}`
  (per-person membership count exactly `k`; at `k = n−1` these ARE the
  complements of singletons, so the canonical instance is a member). The
  fidelity check: the doc's ratification inequalities come out VERBATIM —
  `n` cancels.
-/
import IDT.Composition

namespace BernsteinLogic.IDT

open BernsteinLogic.Imprecise
open Finset

/-! ### Dirac expectation (used by every concrete cell) -/

namespace FinPMFExtra

variable {X : Type*} [Fintype X] [DecidableEq X]

theorem exp_pure (a : X) (f : X → ℚ) : (pure a).exp f = f a := by
  unfold FinPMF.exp
  simp only [pure_prob, ite_mul, one_mul, zero_mul]
  rw [Finset.sum_ite_eq' Finset.univ a f]
  simp

theorem mass_pure (a : X) (E : Finset X) :
    Event.mass (pure a) E = if a ∈ E then 1 else 0 := by
  unfold Event.mass
  simp only [pure_prob]
  rw [Finset.sum_ite_eq' E a fun _ => (1 : ℚ)]

end FinPMFExtra

/-! ### §2.1 — ratifiability and the precise collapse -/

section Ratifiability

variable {X : Type*} [Fintype X] [DecidableEq X]
variable {K : Type*} [Fintype K] [Nonempty K]
variable {Mo I A : Type*} [Fintype Mo] [DecidableEq I] [Fintype A] [Nonempty A]

/-- **Ratifiability** (paper §2.1, posterior form, unrestricted deterministic
deviations): at every observation whose trigger has positive `P̄` — "observed
with positive probability"; with choice-independent triggers this reads off
the stock alone — the incumbent's output at `o` is an MWER choice in the
POST-UPDATE fiber menu `F_{o,π}`. At the precise collapse this is O&C's
Definition-6 compatibility (W668-OC-VERIFICATION claim 1). -/
def Ratifiable (T : Stock X K) (S : Scenario X Mo I A) (π : I → A) : Prop :=
  ∀ o : I, 0 < T.pbar (S.trigger o) →
    IsMWERChoice (T.update (S.trigger o)) (S.fiberMenuDet π o) (π o)

/-- **Ratification is update-invariant** (paper Theorem 1(ii) applied at the
fibers; Corollary 1.2's fiber grade, and §2.6's "updating is provably
innocent"): the posterior membership check of `Ratifiable` equals the prior
one. Holds at ANY stock — this is where selection-vs-stability divergence
(the §2.6 witness below) is proved NOT to be updating's fault. -/
theorem ratifiable_iff_exante {T : Stock X K} {S : Scenario X Mo I A}
    (hLoc : S.Local) (π : I → A) :
    Ratifiable T S π ↔
      ∀ o : I, 0 < T.pbar (S.trigger o) →
        IsMWERChoice T (S.fiberMenuDet π o) (π o) := by
  unfold Ratifiable
  refine forall_congr' fun o => imp_congr_right fun _ => ?_
  exact T.slice_isMWERChoice_iff (S.trigger o) (M := S.fiberMenuDet π o)
    (fun d d' x hx => Scenario.fiberMenuDet_agree_off hLoc π o d d' hx) (π o)

/-- The precise collapse of MWER preference (paper Corollary 1.1(1)): on a
singleton stock, MWER choice = expected-utility maximality — the menu
benchmark is constant across candidates. -/
theorem isMWERChoice_precise_iff (p : FinPMF X) {ι : Type*} [Fintype ι]
    [Nonempty ι] (M : ι → X → ℚ) (i : ι) :
    IsMWERChoice (Stock.precise p) M i ↔ ∀ j, p.exp (M j) ≤ p.exp (M i) := by
  unfold IsMWERChoice
  constructor
  · intro h j
    have h1 := h j
    rw [wregret_precise, wregret_precise] at h1
    unfold regret at h1
    linarith
  · intro h j
    rw [wregret_precise, wregret_precise]
    unfold regret
    have h1 := h j
    linarith

end Ratifiability

/-! ### §2.2 — THEOREM 2: selection = ex-ante optimality; selection ⊆ ratifiable -/

section Theorem2

variable {X : Type*} [Fintype X] [DecidableEq X]
variable {Mo I A : Type*} [Fintype Mo] [DecidableEq I] [Fintype A] [Nonempty A]
variable {P : Type*} [Fintype P] [Nonempty P]

/-- A policy menu, as acts: index `ρ` names the world-utility act of policy
`pol ρ` (paper §2.1's `S_Π` menu). -/
def polMenu (S : Scenario X Mo I A) (pol : P → I → A) : P → X → ℚ :=
  fun ρ => S.uDet (pol ρ)

/-- **Theorem 2(i)** (paper §2.2): at a singleton stock, menu-level MWER
selects exactly the ex-ante expected-utility optima. (Nonemptiness of the
choice set is Basic's `exists_mwerChoice`.) -/
theorem selected_iff_exante_optimal (S : Scenario X Mo I A) (p : FinPMF X)
    (pol : P → I → A) (ρ : P) :
    IsMWERChoice (Stock.precise p) (polMenu S pol) ρ ↔
      ∀ ρ', p.exp (S.uDet (pol ρ')) ≤ p.exp (S.uDet (pol ρ)) :=
  isMWERChoice_precise_iff p (polMenu S pol) ρ

/-- **Theorem 2(ii)** (paper §2.2): at a singleton stock, on a substitution-
closed policy menu, the menu-MWER selection is RATIFIABLE — the selection
principle lands inside the fixed-point set (the O&C Corollary-7 bridge: in
this formalism the inheritance is direct, because each fiber deviation of a
selected policy is again a menu member; the substantive step is Theorem 1's
update-invariance, folded in via `ratifiable_iff_exante`). -/
theorem selected_ratifiable {S : Scenario X Mo I A} (hLoc : S.Local)
    (p : FinPMF X) (pol : P → I → A)
    (hclosed : ∀ ρ o a, ∃ ρ', pol ρ' = Scenario.subst (pol ρ) o a) (ρ : P)
    (hsel : IsMWERChoice (Stock.precise p) (polMenu S pol) ρ) :
    Ratifiable (Stock.precise p) S (pol ρ) := by
  rw [ratifiable_iff_exante hLoc]
  intro o _
  rw [isMWERChoice_precise_iff]
  intro a
  rw [Scenario.fiberMenuDet_self]
  obtain ⟨ρ', hρ'⟩ := hclosed ρ o a
  have h1 := (selected_iff_exante_optimal S p pol ρ).mp hsel ρ'
  show p.exp (S.uDet (Scenario.subst (pol ρ) o a)) ≤ p.exp (S.uDet (pol ρ))
  rw [← hρ']
  exact h1

/-- **Theorem 2(iii)**, the never-selects-a-dominated-policy clause (paper
§2.2, the "never selects a certain-loss policy while Π contains a policy of
higher prior expected value" reading): immediate from (i). -/
theorem selected_not_dominated (S : Scenario X Mo I A) (p : FinPMF X)
    (pol : P → I → A) (ρ : P)
    (hsel : IsMWERChoice (Stock.precise p) (polMenu S pol) ρ) :
    ¬ ∃ ρ', p.exp (S.uDet (pol ρ)) < p.exp (S.uDet (pol ρ')) := by
  rintro ⟨ρ', hρ'⟩
  exact absurd ((selected_iff_exante_optimal S p pol ρ).mp hsel ρ') (not_le.mpr hρ')

end Theorem2

/-! ### §2.3 — the Psy-Kosh parametric family (battery row 11's theorem debt)

Concrete family member: worlds `(c, j) : Bool × Fin n` (`c = true` is the
H-branch with sole decider `j`, mass `q/n`; `c = false` the T-branch with
decider set the cyclic interval `S_j = {j, …, j+k−1}`, mass `(1−q)/n`).
Person `i`'s input is her own identity (`I = Fin n` — inputs distinct by
construction, the battery's encoding); payoffs: H∧all-yea `YH`, T∧all-yea
`YT`, all-nay `NN`, disagreement-among-deciders `0`. The coin bias `q` is
the paper's `h`. Fidelity check: the paper's ratification inequalities come
out verbatim (`n` cancels). -/

namespace PsyKosh

open Scenario

variable (n k : ℕ) [NeZero n]

/-- The cyclic decider interval `S_j = {j, j+1, …, j+k−1}` (mod n), encoded
with `Fin n` offsets of value below `k`. -/
def interval (j : Fin n) : Finset (Fin n) :=
  (univ.filter fun r : Fin n => r.val < k).image fun r => j + r

theorem self_mem_interval (hk : 0 < k) (j : Fin n) : j ∈ interval n k j := by
  refine Finset.mem_image.mpr
    ⟨0, Finset.mem_filter.mpr ⟨Finset.mem_univ _, ?_⟩, add_zero j⟩
  simpa using hk

theorem exists_mem_interval_ne (hk2 : 2 ≤ k) (hn2 : 2 ≤ n) (j i₀ : Fin n) :
    ∃ i ∈ interval n k j, i ≠ i₀ := by
  by_cases hj : j = i₀
  · have h1n : 1 < n := by omega
    refine ⟨j + ⟨1, h1n⟩, Finset.mem_image.mpr
      ⟨⟨1, h1n⟩, Finset.mem_filter.mpr ⟨Finset.mem_univ _, ?_⟩, rfl⟩, ?_⟩
    · show (1:ℕ) < k
      omega
    · subst hj
      intro hcontra
      have h2 : j + (⟨1, h1n⟩ : Fin n) = j + 0 := by
        rw [add_zero]
        exact hcontra
      have h0 : (⟨1, h1n⟩ : Fin n) = 0 := add_left_cancel h2
      have hv : (1 : ℕ) = 0 := congrArg Fin.val h0
      omega
  · exact ⟨j, self_mem_interval n k (by omega) j, hj⟩

/-- Offsets below `k` number exactly `k` (for `k ≤ n`). -/
theorem card_val_lt (hkn : k ≤ n) :
    (univ.filter fun r : Fin n => r.val < k).card = k := by
  have hset : (univ.filter fun r : Fin n => r.val < k)
      = (Finset.range k).attachFin
          (fun m hm => lt_of_lt_of_le (Finset.mem_range.mp hm) hkn) := by
    ext r
    simp [Finset.mem_attachFin, Finset.mem_range]
  rw [hset, Finset.card_attachFin, Finset.card_range]

/-- The counting lemma: each person belongs to exactly `k` cyclic intervals
(the marginal that makes the family's inequalities come out verbatim). -/
theorem card_mem_interval (hkn : k ≤ n) (i₀ : Fin n) :
    (univ.filter fun j => i₀ ∈ interval n k j).card = k := by
  have hset : (univ.filter fun j => i₀ ∈ interval n k j)
      = (univ.filter fun r : Fin n => r.val < k).image fun r => i₀ - r := by
    ext j
    simp only [Finset.mem_filter, Finset.mem_univ, true_and, interval,
      Finset.mem_image]
    constructor
    · rintro ⟨r, hr, hjr⟩
      refine ⟨r, hr, ?_⟩
      show i₀ - r = j
      rw [← hjr]
      abel
    · rintro ⟨r, hr, hjr⟩
      refine ⟨r, hr, ?_⟩
      show j + r = i₀
      rw [← hjr]
      abel
  rw [hset, Finset.card_image_of_injOn, card_val_lt n k hkn]
  intro r₁ _ r₂ _ hrr
  have hrr' : i₀ - r₁ = i₀ - r₂ := hrr
  calc r₁ = i₀ - (i₀ - r₁) := (sub_sub_cancel i₀ r₁).symm
  _ = i₀ - (i₀ - r₂) := by rw [hrr']
  _ = r₂ := sub_sub_cancel i₀ r₂

/-- The decider set of a world. -/
def deciders (x : Bool × Fin n) : Finset (Fin n) :=
  if x.1 then {x.2} else interval n k x.2

/-- The observer moments: (world, decider) pairs. -/
def Mom := {q : (Bool × Fin n) × Fin n // q.2 ∈ deciders n k q.1}

instance : Fintype (Mom n k) := Subtype.fintype _

/-- Moment constructor. -/
def mom (x : Bool × Fin n) (i : Fin n) (hi : i ∈ deciders n k x) : Mom n k :=
  ⟨(x, i), hi⟩

variable (YH YT NN : ℚ)

/-- The Psy-Kosh scenario (battery row 11's encoding, parametric). -/
def psy : Scenario (Bool × Fin n) (Mom n k) (Fin n) Bool where
  world m := m.val.1
  inp m := m.val.2
  payoff x f :=
    if x.1 then
      (if (∀ m : Mom n k, m.val.1 = x → f m = true) then YH else NN)
    else
      (if (∀ m : Mom n k, m.val.1 = x → f m = true) then YT
       else if (∀ m : Mom n k, m.val.1 = x → f m = false) then NN else 0)

theorem psy_local : (psy n k YH YT NN).Local := by
  intro x f g hfg
  have h1 : (∀ m : Mom n k, m.val.1 = x → f m = true)
      ↔ (∀ m : Mom n k, m.val.1 = x → g m = true) :=
    forall_congr' fun m => imp_congr_right fun hm => by rw [hfg m hm]
  have h2 : (∀ m : Mom n k, m.val.1 = x → f m = false)
      ↔ (∀ m : Mom n k, m.val.1 = x → g m = false) :=
    forall_congr' fun m => imp_congr_right fun hm => by rw [hfg m hm]
  simp only [psy]
  simp only [h1, h2]

/-- Payoff evaluation, H-worlds. -/
theorem payoff_H (j : Fin n) (f : Mom n k → Bool) :
    (psy n k YH YT NN).payoff (true, j) f
      = if (∀ m : Mom n k, m.val.1 = (true, j) → f m = true) then YH else NN := by
  simp [psy]

/-- Payoff evaluation, T-worlds. -/
theorem payoff_T (j : Fin n) (f : Mom n k → Bool) :
    (psy n k YH YT NN).payoff (false, j) f
      = if (∀ m : Mom n k, m.val.1 = (false, j) → f m = true) then YT
        else if (∀ m : Mom n k, m.val.1 = (false, j) → f m = false) then NN
        else 0 := by
  simp [psy]

/-! #### uDet values, case by case -/

theorem uDet_val_true_yea {π : Fin n → Bool} {j : Fin n} (hj : π j = true) :
    (psy n k YH YT NN).uDet π (true, j) = YH := by
  show (psy n k YH YT NN).payoff (true, j) _ = YH
  rw [payoff_H, if_pos]
  intro m hm
  have hmem : m.val.2 ∈ deciders n k (true, j) := hm ▸ m.2
  have hij : m.val.2 = j := by simpa [deciders] using hmem
  show π m.val.2 = true
  rw [hij]
  exact hj

theorem uDet_val_true_nay {π : Fin n → Bool} {j : Fin n} (hj : π j = false) :
    (psy n k YH YT NN).uDet π (true, j) = NN := by
  show (psy n k YH YT NN).payoff (true, j) _ = NN
  rw [payoff_H, if_neg]
  intro hall
  have hval : π j = true := hall (mom n k (true, j) j (by simp [deciders])) rfl
  rw [hj] at hval
  exact Bool.false_ne_true hval

theorem uDet_val_false_allyea {π : Fin n → Bool} {j : Fin n}
    (hall : ∀ i ∈ interval n k j, π i = true) :
    (psy n k YH YT NN).uDet π (false, j) = YT := by
  show (psy n k YH YT NN).payoff (false, j) _ = YT
  rw [payoff_T, if_pos]
  intro m hm
  have hmem : m.val.2 ∈ deciders n k (false, j) := hm ▸ m.2
  show π m.val.2 = true
  exact hall m.val.2 (by simpa [deciders] using hmem)

theorem uDet_val_false_allnay (hk : 0 < k) {π : Fin n → Bool} {j : Fin n}
    (hall : ∀ i ∈ interval n k j, π i = false) :
    (psy n k YH YT NN).uDet π (false, j) = NN := by
  show (psy n k YH YT NN).payoff (false, j) _ = NN
  rw [payoff_T, if_neg, if_pos]
  · intro m hm
    have hmem : m.val.2 ∈ deciders n k (false, j) := hm ▸ m.2
    show π m.val.2 = false
    exact hall m.val.2 (by simpa [deciders] using hmem)
  · intro hy
    have h1 : π j = true := hy
      (mom n k (false, j) j (by simpa [deciders] using self_mem_interval n k hk j)) rfl
    have h2 : π j = false := hall j (self_mem_interval n k hk j)
    rw [h2] at h1
    exact Bool.false_ne_true h1

theorem uDet_val_false_mixed {π : Fin n → Bool} {j : Fin n}
    (hyes : ∃ i ∈ interval n k j, π i = true)
    (hno : ∃ i ∈ interval n k j, π i = false) :
    (psy n k YH YT NN).uDet π (false, j) = 0 := by
  obtain ⟨iy, hiy, hy⟩ := hyes
  obtain ⟨im, him, hm⟩ := hno
  show (psy n k YH YT NN).payoff (false, j) _ = 0
  rw [payoff_T, if_neg, if_neg]
  · intro hall
    have h1 : π iy = false :=
      hall (mom n k (false, j) iy (by simpa [deciders] using hiy)) rfl
    rw [hy] at h1
    exact Bool.noConfusion h1
  · intro hall
    have h1 : π im = true :=
      hall (mom n k (false, j) im (by simpa [deciders] using him)) rfl
    rw [hm] at h1
    exact Bool.false_ne_true h1

/-! #### The world measure and expectation splitting -/

variable (q : ℚ)

/-- The world measure: H-worlds `q/n` each, T-worlds `(1−q)/n` each. -/
def pmf (q0 : 0 ≤ q) (q1 : q ≤ 1) : FinPMF (Bool × Fin n) where
  prob x := if x.1 then q / n else (1 - q) / n
  nonneg x := by
    have hn : (0:ℚ) ≤ n := Nat.cast_nonneg n
    by_cases hx : x.1
    · simp only [hx, if_true]
      exact div_nonneg q0 hn
    · simp only [hx, if_false]
      exact div_nonneg (by linarith) hn
  total := by
    have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
    rw [Fintype.sum_prod_type, Fintype.sum_bool]
    show (∑ _j : Fin n, q / n) + (∑ _j : Fin n, (1 - q) / n) = 1
    simp only [Finset.sum_const, Finset.card_univ, Fintype.card_fin,
      nsmul_eq_mul]
    field_simp
    ring

theorem exp_split (q0 : 0 ≤ q) (q1 : q ≤ 1) (u : Bool × Fin n → ℚ) :
    (pmf n q q0 q1).exp u
      = q / n * ∑ j, u (true, j) + (1 - q) / n * ∑ j, u (false, j) := by
  show (∑ x : Bool × Fin n, (pmf n q q0 q1).prob x * u x) = _
  rw [Fintype.sum_prod_type, Fintype.sum_bool]
  show (∑ j : Fin n, q / n * u (true, j)) + (∑ j : Fin n, (1 - q) / n * u (false, j)) = _
  rw [Finset.mul_sum, Finset.mul_sum]

/-! #### The four expected utilities -/

theorem EU_yea (q0 : 0 ≤ q) (q1 : q ≤ 1) :
    (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true)
      = q * YH + (1 - q) * YT := by
  have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
  rw [exp_split,
    Finset.sum_congr rfl fun j _ => uDet_val_true_yea n k YH YT NN (π := fun _ => true) rfl,
    Finset.sum_congr rfl fun j _ =>
      uDet_val_false_allyea n k YH YT NN (π := fun _ => true) (fun _ _ => rfl)]
  simp only [Finset.sum_const, Finset.card_univ, Fintype.card_fin, nsmul_eq_mul]
  field_simp

theorem EU_nay (hk : 0 < k) (q0 : 0 ≤ q) (q1 : q ≤ 1) :
    (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false) = NN := by
  have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
  rw [exp_split,
    Finset.sum_congr rfl fun j _ => uDet_val_true_nay n k YH YT NN (π := fun _ => false) rfl,
    Finset.sum_congr rfl fun j _ =>
      uDet_val_false_allnay n k YH YT NN hk (π := fun _ => false) (fun _ _ => rfl)]
  simp only [Finset.sum_const, Finset.card_univ, Fintype.card_fin, nsmul_eq_mul]
  field_simp
  ring

theorem EU_yea_dev (hk2 : 2 ≤ k) (hkn : k ≤ n) (q0 : 0 ≤ q) (q1 : q ≤ 1)
    (i₀ : Fin n) :
    (pmf n q q0 q1).exp
        ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => true) i₀ false))
      = q / n * (((n:ℚ) - 1) * YH + NN) + (1 - q) / n * (((n:ℚ) - (k:ℚ)) * YT) := by
  have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
  have hn2 : 2 ≤ n := le_trans hk2 hkn
  have hH : ∀ j : Fin n,
      (psy n k YH YT NN).uDet (Scenario.subst (fun _ => true) i₀ false) (true, j)
        = if j = i₀ then NN else YH := by
    intro j
    by_cases hj : j = i₀
    · rw [if_pos hj]
      exact uDet_val_true_nay n k YH YT NN (by simp [Scenario.subst, hj])
    · rw [if_neg hj]
      exact uDet_val_true_yea n k YH YT NN (by simp [Scenario.subst, hj])
  have hT : ∀ j : Fin n,
      (psy n k YH YT NN).uDet (Scenario.subst (fun _ => true) i₀ false) (false, j)
        = if i₀ ∈ interval n k j then 0 else YT := by
    intro j
    by_cases hmem : i₀ ∈ interval n k j
    · rw [if_pos hmem]
      obtain ⟨i, hi, hine⟩ := exists_mem_interval_ne n k hk2 hn2 j i₀
      exact uDet_val_false_mixed n k YH YT NN
        ⟨i, hi, by simp [Scenario.subst, hine]⟩
        ⟨i₀, hmem, by simp [Scenario.subst]⟩
    · rw [if_neg hmem]
      refine uDet_val_false_allyea n k YH YT NN fun i hi => ?_
      have hine : i ≠ i₀ := fun he => hmem (he ▸ hi)
      simp [Scenario.subst, hine]
  rw [exp_split, Finset.sum_congr rfl fun j _ => hH j,
    Finset.sum_congr rfl fun j _ => hT j]
  have hHsum : ∑ j : Fin n, (if j = i₀ then NN else YH)
      = (n:ℚ) * YH + (NN - YH) := by
    have hpt : ∀ j : Fin n, (if j = i₀ then NN else YH)
        = YH + (if j = i₀ then NN - YH else 0) := by
      intro j
      by_cases hj : j = i₀ <;> simp [hj]
    rw [Finset.sum_congr rfl fun j _ => hpt j, Finset.sum_add_distrib]
    simp only [Finset.sum_const, Finset.card_univ, Fintype.card_fin, nsmul_eq_mul]
    rw [Finset.sum_ite_eq' Finset.univ i₀ fun _ => NN - YH]
    simp
  have hTsum : ∑ j : Fin n, (if i₀ ∈ interval n k j then (0:ℚ) else YT)
      = ((n:ℚ) - (k:ℚ)) * YT := by
    rw [Finset.sum_ite, Finset.sum_const_zero, Finset.sum_const, zero_add,
      nsmul_eq_mul]
    have hcards := Finset.card_filter_add_card_filter_not
      (s := (univ : Finset (Fin n))) (fun j => i₀ ∈ interval n k j)
    rw [card_mem_interval n k hkn i₀, Finset.card_univ, Fintype.card_fin] at hcards
    have hcard2 : (univ.filter fun j => i₀ ∉ interval n k j).card = n - k := by omega
    rw [hcard2, Nat.cast_sub hkn]
  rw [hHsum, hTsum]
  ring

theorem EU_nay_dev (hk2 : 2 ≤ k) (hkn : k ≤ n) (q0 : 0 ≤ q) (q1 : q ≤ 1)
    (i₀ : Fin n) :
    (pmf n q q0 q1).exp
        ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => false) i₀ true))
      = q / n * (((n:ℚ) - 1) * NN + YH) + (1 - q) / n * (((n:ℚ) - (k:ℚ)) * NN) := by
  have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
  have hn2 : 2 ≤ n := le_trans hk2 hkn
  have hH : ∀ j : Fin n,
      (psy n k YH YT NN).uDet (Scenario.subst (fun _ => false) i₀ true) (true, j)
        = if j = i₀ then YH else NN := by
    intro j
    by_cases hj : j = i₀
    · rw [if_pos hj]
      exact uDet_val_true_yea n k YH YT NN (by simp [Scenario.subst, hj])
    · rw [if_neg hj]
      exact uDet_val_true_nay n k YH YT NN (by simp [Scenario.subst, hj])
  have hT : ∀ j : Fin n,
      (psy n k YH YT NN).uDet (Scenario.subst (fun _ => false) i₀ true) (false, j)
        = if i₀ ∈ interval n k j then 0 else NN := by
    intro j
    by_cases hmem : i₀ ∈ interval n k j
    · rw [if_pos hmem]
      obtain ⟨i, hi, hine⟩ := exists_mem_interval_ne n k hk2 hn2 j i₀
      exact uDet_val_false_mixed n k YH YT NN
        ⟨i₀, hmem, by simp [Scenario.subst]⟩
        ⟨i, hi, by simp [Scenario.subst, hine]⟩
    · rw [if_neg hmem]
      refine uDet_val_false_allnay n k YH YT NN (by omega) fun i hi => ?_
      have hine : i ≠ i₀ := fun he => hmem (he ▸ hi)
      simp [Scenario.subst, hine]
  rw [exp_split, Finset.sum_congr rfl fun j _ => hH j,
    Finset.sum_congr rfl fun j _ => hT j]
  have hHsum : ∑ j : Fin n, (if j = i₀ then YH else NN)
      = (n:ℚ) * NN + (YH - NN) := by
    have hpt : ∀ j : Fin n, (if j = i₀ then YH else NN)
        = NN + (if j = i₀ then YH - NN else 0) := by
      intro j
      by_cases hj : j = i₀ <;> simp [hj]
    rw [Finset.sum_congr rfl fun j _ => hpt j, Finset.sum_add_distrib]
    simp only [Finset.sum_const, Finset.card_univ, Fintype.card_fin, nsmul_eq_mul]
    rw [Finset.sum_ite_eq' Finset.univ i₀ fun _ => YH - NN]
    simp
  have hTsum : ∑ j : Fin n, (if i₀ ∈ interval n k j then (0:ℚ) else NN)
      = ((n:ℚ) - (k:ℚ)) * NN := by
    rw [Finset.sum_ite, Finset.sum_const_zero, Finset.sum_const, zero_add,
      nsmul_eq_mul]
    have hcards := Finset.card_filter_add_card_filter_not
      (s := (univ : Finset (Fin n))) (fun j => i₀ ∈ interval n k j)
    rw [card_mem_interval n k hkn i₀, Finset.card_univ, Fintype.card_fin] at hcards
    have hcard2 : (univ.filter fun j => i₀ ∉ interval n k j).card = n - k := by omega
    rw [hcard2, Nat.cast_sub hkn]
  rw [hHsum, hTsum]
  ring

/-! #### The gate: every trigger has positive `P̄` -/

theorem pbar_trigger_pos (hk : 0 < k) (q0 : 0 ≤ q) (q1 : q ≤ 1) (i₀ : Fin n) :
    0 < (Stock.precise (pmf n q q0 q1)).pbar ((psy n k YH YT NN).trigger i₀) := by
  have hn0 : (0:ℚ) < n := by
    exact_mod_cast Nat.pos_of_ne_zero (NeZero.ne n)
  rw [Stock.pbar_precise]
  have hmem1 : (true, i₀) ∈ (psy n k YH YT NN).trigger i₀ :=
    ((psy n k YH YT NN).mem_trigger i₀ (true, i₀)).mpr
      ⟨mom n k (true, i₀) i₀ (by simp [deciders]), rfl, rfl⟩
  have hmem2 : (false, i₀) ∈ (psy n k YH YT NN).trigger i₀ :=
    ((psy n k YH YT NN).mem_trigger i₀ (false, i₀)).mpr
      ⟨mom n k (false, i₀) i₀
        (by simpa [deciders] using self_mem_interval n k hk i₀), rfl, rfl⟩
  have hpair : ({(true, i₀), (false, i₀)} : Finset (Bool × Fin n))
      ⊆ (psy n k YH YT NN).trigger i₀ := by
    intro x hx
    rcases Finset.mem_insert.mp hx with h | h
    · rw [h]; exact hmem1
    · rw [Finset.mem_singleton.mp h]; exact hmem2
  have hsum : ∑ x ∈ ({(true, i₀), (false, i₀)} : Finset (Bool × Fin n)),
        (pmf n q q0 q1).prob x
      ≤ Event.mass (pmf n q q0 q1) ((psy n k YH YT NN).trigger i₀) :=
    Finset.sum_le_sum_of_subset_of_nonneg hpair
      (fun x _ _ => (pmf n q q0 q1).nonneg x)
  have hne : ((true, i₀) : Bool × Fin n) ≠ (false, i₀) := by simp
  rw [Finset.sum_pair hne] at hsum
  have hval : (pmf n q q0 q1).prob (true, i₀) + (pmf n q q0 q1).prob (false, i₀)
      = 1 / n := by
    show q / n + (1 - q) / n = 1 / n
    field_simp
    ring
  rw [hval] at hsum
  exact lt_of_lt_of_le (by positivity) hsum

/-! #### Ratification, characterized (the doc's verbatim inequalities) -/

theorem yea_ratifiable_iff (hk2 : 2 ≤ k) (hkn : k ≤ n) (q0 : 0 ≤ q) (q1 : q ≤ 1) :
    Ratifiable (Stock.precise (pmf n q q0 q1)) (psy n k YH YT NN) (fun _ => true)
      ↔ q * NN ≤ q * YH + (1 - q) * (k:ℚ) * YT := by
  have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
  have hn0 : (0:ℚ) < n := by exact_mod_cast Nat.pos_of_ne_zero (NeZero.ne n)
  have key : ∀ i₀ : Fin n,
      (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true)
        - (pmf n q q0 q1).exp
            ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => true) i₀ false))
      = (q * YH + (1 - q) * (k:ℚ) * YT - q * NN) / n := by
    intro i₀
    rw [EU_yea, EU_yea_dev n k YH YT NN q hk2 hkn q0 q1 i₀]
    field_simp
    ring
  rw [ratifiable_iff_exante (psy_local n k YH YT NN)]
  constructor
  · intro H
    have hg := H 0 (pbar_trigger_pos n k YH YT NN q (by omega) q0 q1 0)
    have hle := (isMWERChoice_precise_iff _ _ _).mp hg false
    rw [Scenario.fiberMenuDet_self] at hle
    have hle2 : (pmf n q q0 q1).exp
        ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => true) 0 false))
        ≤ (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true) := hle
    have hkey := key 0
    have hpos : 0 ≤ (q * YH + (1 - q) * (k:ℚ) * YT - q * NN) / n := by
      rw [← hkey]
      linarith
    have := (le_div_iff₀ hn0).mp hpos
    linarith
  · intro H i₀ _
    rw [isMWERChoice_precise_iff]
    intro a
    rw [Scenario.fiberMenuDet_self]
    cases a with
    | true =>
      have hsubst : Scenario.subst (fun _ : Fin n => true) i₀ true
          = fun _ : Fin n => true := Scenario.subst_self _ i₀
      show (pmf n q q0 q1).exp
          ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => true) i₀ true)) ≤ _
      rw [hsubst]
    | false =>
      show (pmf n q q0 q1).exp
          ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => true) i₀ false)) ≤ _
      have hkey := key i₀
      have hpos : 0 ≤ (q * YH + (1 - q) * (k:ℚ) * YT - q * NN) / n :=
        div_nonneg (by linarith) hn0.le
      linarith

theorem nay_ratifiable_iff (hk2 : 2 ≤ k) (hkn : k ≤ n) (q0 : 0 ≤ q) (q1 : q ≤ 1) :
    Ratifiable (Stock.precise (pmf n q q0 q1)) (psy n k YH YT NN) (fun _ => false)
      ↔ q * YH ≤ NN * (q + (1 - q) * (k:ℚ)) := by
  have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
  have hn0 : (0:ℚ) < n := by exact_mod_cast Nat.pos_of_ne_zero (NeZero.ne n)
  have key : ∀ i₀ : Fin n,
      (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false)
        - (pmf n q q0 q1).exp
            ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => false) i₀ true))
      = (NN * (q + (1 - q) * (k:ℚ)) - q * YH) / n := by
    intro i₀
    rw [EU_nay n k YH YT NN q (by omega) q0 q1,
      EU_nay_dev n k YH YT NN q hk2 hkn q0 q1 i₀]
    field_simp
    ring
  rw [ratifiable_iff_exante (psy_local n k YH YT NN)]
  constructor
  · intro H
    have hg := H 0 (pbar_trigger_pos n k YH YT NN q (by omega) q0 q1 0)
    have hle := (isMWERChoice_precise_iff _ _ _).mp hg true
    rw [Scenario.fiberMenuDet_self] at hle
    have hle2 : (pmf n q q0 q1).exp
        ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => false) 0 true))
        ≤ (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false) := hle
    have hkey := key 0
    have hpos : 0 ≤ (NN * (q + (1 - q) * (k:ℚ)) - q * YH) / n := by
      rw [← hkey]
      linarith
    have := (le_div_iff₀ hn0).mp hpos
    linarith
  · intro H i₀ _
    rw [isMWERChoice_precise_iff]
    intro a
    rw [Scenario.fiberMenuDet_self]
    cases a with
    | false =>
      have hsubst : Scenario.subst (fun _ : Fin n => false) i₀ false
          = fun _ : Fin n => false := Scenario.subst_self _ i₀
      show (pmf n q q0 q1).exp
          ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => false) i₀ false)) ≤ _
      rw [hsubst]
    | true =>
      show (pmf n q q0 q1).exp
          ((psy n k YH YT NN).uDet (Scenario.subst (fun _ => false) i₀ true)) ≤ _
      have hkey := key i₀
      have hpos : 0 ≤ (NN * (q + (1 - q) * (k:ℚ)) - q * YH) / n :=
        div_nonneg (by linarith) hn0.le
      linarith

/-! #### Selection on the symmetric menu, the family theorem, the full menu -/

/-- The symmetric (constant-policy) menu. -/
def symPol : Bool → Fin n → Bool := fun b _ => b

theorem selection_yea_iff (hk : 0 < k) (q0 : 0 ≤ q) (q1 : q ≤ 1) :
    IsMWERChoice (Stock.precise (pmf n q q0 q1))
        (polMenu (psy n k YH YT NN) (symPol n)) true
      ↔ NN ≤ q * YH + (1 - q) * YT := by
  rw [isMWERChoice_precise_iff]
  constructor
  · intro H
    have hle : (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false)
        ≤ (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true) := H false
    rw [EU_yea, EU_nay n k YH YT NN q hk q0 q1] at hle
    exact hle
  · intro H j
    cases j with
    | true => exact le_refl _
    | false =>
      show (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false)
        ≤ (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true)
      rw [EU_yea, EU_nay n k YH YT NN q hk q0 q1]
      exact H

theorem selection_nay_iff (hk : 0 < k) (q0 : 0 ≤ q) (q1 : q ≤ 1) :
    IsMWERChoice (Stock.precise (pmf n q q0 q1))
        (polMenu (psy n k YH YT NN) (symPol n)) false
      ↔ q * YH + (1 - q) * YT ≤ NN := by
  rw [isMWERChoice_precise_iff]
  constructor
  · intro H
    have hle : (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true)
        ≤ (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false) := H true
    rw [EU_yea, EU_nay n k YH YT NN q hk q0 q1] at hle
    exact hle
  · intro H j
    cases j with
    | false => exact le_refl _
    | true =>
      show (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true)
        ≤ (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false)
      rw [EU_yea, EU_nay n k YH YT NN q hk q0 q1]
      exact H

/-- **The family theorem** (paper §2.3): across the whole parametric family,
the symmetric-menu selection is ratifiable w.r.t. UNRESTRICTED fiber
deviations — the selection principle always lands on a genuine fixed point.
(The inequality chase of the paper, verbatim.) -/
theorem family_selection_ratifiable (hk2 : 2 ≤ k) (hkn : k ≤ n)
    (hYH : 0 ≤ YH) (hYT : 0 ≤ YT) (hNN : 0 ≤ NN) (q0 : 0 ≤ q) (q1 : q ≤ 1)
    (b : Bool)
    (hsel : IsMWERChoice (Stock.precise (pmf n q q0 q1))
      (polMenu (psy n k YH YT NN) (symPol n)) b) :
    Ratifiable (Stock.precise (pmf n q q0 q1)) (psy n k YH YT NN) (symPol n b) := by
  have hk : 0 < k := by omega
  have hk1 : (1:ℚ) ≤ (k:ℚ) := by exact_mod_cast hk
  cases b with
  | true =>
    have hs := (selection_yea_iff n k YH YT NN q hk q0 q1).mp hsel
    show Ratifiable _ _ (fun _ => true)
    rw [yea_ratifiable_iff n k YH YT NN q hk2 hkn q0 q1]
    have e1 : 0 ≤ (1 - q) * ((k:ℚ) - 1) * YT :=
      mul_nonneg (mul_nonneg (by linarith) (by linarith)) hYT
    have e2 : 0 ≤ (1 - q) * NN := mul_nonneg (by linarith) hNN
    nlinarith [hs, e1, e2]
  | false =>
    have hs := (selection_nay_iff n k YH YT NN q hk q0 q1).mp hsel
    show Ratifiable _ _ (fun _ => false)
    rw [nay_ratifiable_iff n k YH YT NN q hk2 hkn q0 q1]
    have e1 : 0 ≤ (1 - q) * ((k:ℚ) - 1) * NN :=
      mul_nonneg (mul_nonneg (by linarith) (by linarith)) hNN
    have e2 : 0 ≤ (1 - q) * YT := mul_nonneg (by linarith) hYT
    nlinarith [hs, e1, e2]

/-- **The family-wide full-menu check** (paper §2.3's scope note, §3(2)'s
"family-wide full-menu check" — here STRENGTHENED from a sweep cell to a
theorem): over the FULL deterministic policy menu, every policy is weakly
dominated by the better constant — `EU(π)` is at most the convex combination
`(a/n)·EU(yea) + ((n−a)/n)·EU(nay)` (`a` = number of yea-voters), because a
full-yea interval starts at a yea position and a full-nay interval at a nay
position. -/
theorem full_menu_dominated (hk : 0 < k) (hYT : 0 ≤ YT) (hNN : 0 ≤ NN)
    (q0 : 0 ≤ q) (q1 : q ≤ 1) (π : Fin n → Bool) :
    (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet π)
      ≤ max ((pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true))
          ((pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false)) := by
  have hn : (n:ℚ) ≠ 0 := Nat.cast_ne_zero.mpr (NeZero.ne n)
  have hn0 : (0:ℚ) < n := by exact_mod_cast Nat.pos_of_ne_zero (NeZero.ne n)
  set a : ℕ := (univ.filter fun j => π j = true).card with ha
  have haleN : a ≤ n := by
    calc a ≤ (univ : Finset (Fin n)).card := Finset.card_filter_le _ _
    _ = n := by rw [Finset.card_univ, Fintype.card_fin]
  have hcards := Finset.card_filter_add_card_filter_not
    (s := (univ : Finset (Fin n))) (fun j => π j = true)
  rw [Finset.card_univ, Fintype.card_fin, ← ha] at hcards
  have hcardneg : (univ.filter fun j => ¬ π j = true).card = n - a := by omega
  have hH : ∀ j : Fin n, (psy n k YH YT NN).uDet π (true, j)
      = if π j = true then YH else NN := by
    intro j
    by_cases hj : π j = true
    · rw [if_pos hj]
      exact uDet_val_true_yea n k YH YT NN hj
    · rw [if_neg hj]
      refine uDet_val_true_nay n k YH YT NN ?_
      cases hb : π j
      · rfl
      · exact absurd hb hj
  have hT : ∀ j : Fin n, (psy n k YH YT NN).uDet π (false, j)
      = if (∀ i ∈ interval n k j, π i = true) then YT
        else if (∀ i ∈ interval n k j, π i = false) then NN else 0 := by
    intro j
    by_cases hy : ∀ i ∈ interval n k j, π i = true
    · rw [if_pos hy]
      exact uDet_val_false_allyea n k YH YT NN hy
    · rw [if_neg hy]
      by_cases hna : ∀ i ∈ interval n k j, π i = false
      · rw [if_pos hna]
        exact uDet_val_false_allnay n k YH YT NN hk hna
      · rw [if_neg hna]
        push_neg at hy hna
        obtain ⟨iy, hiy, hy'⟩ := hy
        obtain ⟨im, him, hm'⟩ := hna
        refine uDet_val_false_mixed n k YH YT NN
          ⟨im, him, ?_⟩ ⟨iy, hiy, ?_⟩
        · cases hb : π im
          · exact absurd hb hm'
          · rfl
        · cases hb : π iy
          · rfl
          · exact absurd hb hy'
  rw [exp_split, Finset.sum_congr rfl fun j _ => hH j,
    Finset.sum_congr rfl fun j _ => hT j,
    EU_yea, EU_nay n k YH YT NN q hk q0 q1]
  have hHsum : ∑ j : Fin n, (if π j = true then YH else NN)
      = (a:ℚ) * YH + ((n:ℚ) - (a:ℚ)) * NN := by
    rw [Finset.sum_ite]
    simp only [Finset.sum_const, nsmul_eq_mul]
    rw [← ha, hcardneg, Nat.cast_sub haleN]
  have hTsum : ∑ j : Fin n, (if (∀ i ∈ interval n k j, π i = true) then YT
        else if (∀ i ∈ interval n k j, π i = false) then NN else 0)
      ≤ (a:ℚ) * YT + ((n:ℚ) - (a:ℚ)) * NN := by
    have hA : (univ.filter fun j => ∀ i ∈ interval n k j, π i = true).card ≤ a := by
      rw [ha]
      apply Finset.card_le_card
      intro j hj
      simp only [Finset.mem_filter, Finset.mem_univ, true_and] at hj ⊢
      exact hj j (self_mem_interval n k hk j)
    have hB : ((univ.filter fun j => ¬ ∀ i ∈ interval n k j, π i = true).filter
        fun j => ∀ i ∈ interval n k j, π i = false).card ≤ n - a := by
      rw [← hcardneg]
      apply Finset.card_le_card
      intro j hj
      simp only [Finset.mem_filter, Finset.mem_univ, true_and] at hj ⊢
      rw [hj.2 j (self_mem_interval n k hk j)]
      simp
    rw [Finset.sum_ite, Finset.sum_ite]
    simp only [Finset.sum_const, Finset.sum_const_zero, nsmul_eq_mul, add_zero]
    have hAq : ((univ.filter fun j => ∀ i ∈ interval n k j, π i = true).card : ℚ)
        ≤ (a:ℚ) := by exact_mod_cast hA
    have hBq : (((univ.filter fun j => ¬ ∀ i ∈ interval n k j, π i = true).filter
        fun j => ∀ i ∈ interval n k j, π i = false).card : ℚ)
        ≤ (n:ℚ) - (a:ℚ) := by
      have : (((univ.filter fun j => ¬ ∀ i ∈ interval n k j, π i = true).filter
          fun j => ∀ i ∈ interval n k j, π i = false).card : ℚ) ≤ ((n - a : ℕ) : ℚ) := by
        exact_mod_cast hB
      rwa [Nat.cast_sub haleN] at this
    have c1 := mul_le_mul_of_nonneg_right hAq hYT
    have c2 := mul_le_mul_of_nonneg_right hBq hNN
    linarith
  rw [hHsum]
  set M1 := q * YH + (1 - q) * YT with hM1
  set M2 := NN with hM2
  have key : q / n * ((a:ℚ) * YH + ((n:ℚ) - (a:ℚ)) * M2)
      + (1 - q) / n * ((a:ℚ) * YT + ((n:ℚ) - (a:ℚ)) * M2)
      = ((a:ℚ)/n) * M1 + (((n:ℚ) - (a:ℚ))/n) * M2 := by
    rw [hM1, hM2]
    field_simp
    ring
  have hfrac1 : 0 ≤ (a:ℚ)/n := div_nonneg (Nat.cast_nonneg a) hn0.le
  have hfrac2 : 0 ≤ ((n:ℚ) - (a:ℚ))/n := by
    have hc : (a:ℚ) ≤ (n:ℚ) := by exact_mod_cast haleN
    exact div_nonneg (by linarith) hn0.le
  have hsum1 : (a:ℚ)/n + ((n:ℚ) - (a:ℚ))/n = 1 := by
    field_simp
    ring
  have c1 := mul_le_mul_of_nonneg_left (le_max_left M1 M2) hfrac1
  have c2 := mul_le_mul_of_nonneg_left (le_max_right M1 M2) hfrac2
  have hfac : 0 ≤ (1 - q)/n := div_nonneg (by linarith) hn0.le
  have hstep := mul_le_mul_of_nonneg_left hTsum hfac
  have hfinal : ((a:ℚ)/n) * M1 + (((n:ℚ) - (a:ℚ))/n) * M2 ≤ max M1 M2 := by
    have hmul : ((a:ℚ)/n + ((n:ℚ) - (a:ℚ))/n) * max M1 M2 = 1 * max M1 M2 := by
      rw [hsum1]
    nlinarith [c1, c2, hmul]
  nlinarith [key, hstep, hfinal]

/-- Full-menu corollary: the full-menu MWER selection can always be taken at
a constant policy, and that constant is ratifiable (via the family theorem —
the doc's "full circle": selection = ex-ante optimum = a genuine fixed
point). -/
theorem full_menu_const_choice_ratifiable (hk2 : 2 ≤ k) (hkn : k ≤ n)
    (hYH : 0 ≤ YH) (hYT : 0 ≤ YT) (hNN : 0 ≤ NN) (q0 : 0 ≤ q) (q1 : q ≤ 1) :
    ∃ b : Bool,
      IsMWERChoice (Stock.precise (pmf n q q0 q1))
        (polMenu (psy n k YH YT NN) (fun π : Fin n → Bool => π)) (symPol n b)
      ∧ Ratifiable (Stock.precise (pmf n q q0 q1)) (psy n k YH YT NN)
          (symPol n b) := by
  have hk : 0 < k := by omega
  have hdom := full_menu_dominated n k YH YT NN q hk hYT hNN q0 q1
  by_cases hbc : (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => false)
      ≤ (pmf n q q0 q1).exp ((psy n k YH YT NN).uDet fun _ => true)
  · refine ⟨true, ?_, ?_⟩
    · rw [isMWERChoice_precise_iff]
      intro π
      have := hdom π
      rw [max_eq_left hbc] at this
      exact this
    · -- yea is symmetric-menu-selected, hence ratifiable by the family theorem
      refine family_selection_ratifiable n k YH YT NN q hk2 hkn hYH hYT hNN q0 q1
        true ?_
      rw [isMWERChoice_precise_iff]
      intro j
      cases j with
      | true => exact le_refl _
      | false => exact hbc
  · have hbc' := (not_le.mp hbc).le
    refine ⟨false, ?_, ?_⟩
    · rw [isMWERChoice_precise_iff]
      intro π
      have := hdom π
      rw [max_eq_right hbc'] at this
      exact this
    · refine family_selection_ratifiable n k YH YT NN q hk2 hkn hYH hYT hNN q0 q1
        false ?_
      rw [isMWERChoice_precise_iff]
      intro j
      cases j with
      | false => exact le_refl _
      | true => exact hbc'

end PsyKosh

/-! ### The canonical Psy-Kosh instance (n = 10, k = 9, q = 1/2; payoffs
100/1000/700): the trap region is NONEMPTY — both constants are ratifiable
fixed points, and the selection principle picks nay = the ex-ante optimum,
strictly (the battery row 11 numbers, `550 < 700`). -/

section Canonical

theorem canonical_trap :
    Ratifiable (Stock.precise (PsyKosh.pmf 10 (1/2) (by norm_num) (by norm_num)))
        (PsyKosh.psy 10 9 100 1000 700) (fun _ => true)
    ∧ Ratifiable (Stock.precise (PsyKosh.pmf 10 (1/2) (by norm_num) (by norm_num)))
        (PsyKosh.psy 10 9 100 1000 700) (fun _ => false)
    ∧ IsMWERChoice (Stock.precise (PsyKosh.pmf 10 (1/2) (by norm_num) (by norm_num)))
        (polMenu (PsyKosh.psy 10 9 100 1000 700) (PsyKosh.symPol 10)) false
    ∧ ¬ IsMWERChoice (Stock.precise (PsyKosh.pmf 10 (1/2) (by norm_num) (by norm_num)))
        (polMenu (PsyKosh.psy 10 9 100 1000 700) (PsyKosh.symPol 10)) true := by
  refine ⟨?_, ?_, ?_, ?_⟩
  · rw [PsyKosh.yea_ratifiable_iff 10 9 100 1000 700 (1/2) (by norm_num)
      (by norm_num) (by norm_num) (by norm_num)]
    push_cast
    norm_num
  · rw [PsyKosh.nay_ratifiable_iff 10 9 100 1000 700 (1/2) (by norm_num)
      (by norm_num) (by norm_num) (by norm_num)]
    push_cast
    norm_num
  · rw [PsyKosh.selection_nay_iff 10 9 100 1000 700 (1/2) (by norm_num)
      (by norm_num) (by norm_num)]
    norm_num
  · rw [PsyKosh.selection_yea_iff 10 9 100 1000 700 (1/2) (by norm_num)
      (by norm_num) (by norm_num)]
    norm_num

end Canonical

/-! ### §2.6 — the imprecise fence: the preregistered sub-claim that came out
FALSE (first-class cargo; mechanization debt §3(5))

The exact-ℚ 2×2×2 witness: two worlds (`w₁ = true`, `w₂ = false`), two
inputs (`o₁ = true`, `o₂ = false`, both realized in both worlds — FNC
vacuous, everything prior-visible), two options per input (at `o₁`:
`a = true`, `b = false`; at `o₂`: `c = true`, `d = false`), additive local
utility per the paper's `f`/`g` tables. At the weighted stock `{(δ_{w₁},1),
(δ_{w₂},t)}`:

* `t = 1` (the witness): the UNIQUE global menu-MWER optimum `(a,c)` is
  fiber-UNSTABLE — `S_Π ∩ Rat_Π = ∅` — while `(b,c)` is coordinatewise
  stable yet globally suboptimal (both directions of the R8 global/local
  gap in one instance). Updating is provably innocent
  (`ratifiable_iff_exante` holds at any stock).
* `t = 1/100` (the singleton-limit control, §3(5)'s mutant): the unique
  optimum becomes `(a,d)` = measure 1's EU-optimum, and it IS ratifiable —
  stability restores as the second weight vanishes, exhibiting Theorem 2
  as the boundary case.
* the `Threshold` section (the α₂→0 sweep, §3(5) upgraded from spot-checks
  to a limit statement; prereg
  `hearth/maitria/prereg/W668-ALPHA2-SWEEP-PREREG.md`): stability restores
  at EXACTLY `t₀ = 1/9` — for all `0 ≤ t < 1/9` every global MWER choice
  is `(a,d)` and ratifiable (`stability_restored`), while on `(1/9, 1/4)`
  the unique choice `(b,d)` is NOT ratifiable (`sharpness`) — so
  restoration is NON-monotone in `t`: two disjoint instability windows,
  `(1/9, 1/4)` and `(4/5, 5/4)`, with the `t = 1` witness in the second.

Cells are crunched through the `le_antisymm` API + `norm_num` (Basic's
design note: `by decide` cannot evaluate `ℚ` at this pin); the parametric
section trades `norm_num` endgames for `linarith` under symbolic `t`. -/

namespace SelWitness

open Scenario

/-- The witness scenario: worlds `Bool`, moments `(world, input)` pairs,
inputs `Bool`, options `Bool`. Utility tables (paper §2.6):
`f₁(a)=1, f₁(b)=0; f₂(a)=0, f₂(b)=4; g₁(c)=0, g₁(d)=4; g₂(c)=5, g₂(d)=0`. -/
def wit : Scenario Bool (Bool × Bool) Bool Bool where
  world m := m.1
  inp m := m.2
  payoff x f :=
    (if x then (if f (x, true) then (1:ℚ) else 0) else (if f (x, true) then 0 else 4))
      + (if x then (if f (x, false) then (0:ℚ) else 4) else (if f (x, false) then 5 else 0))

theorem wit_local : wit.Local := by
  intro x f g hfg
  show (if x then (if f (x, true) then (1:ℚ) else 0) else (if f (x, true) then 0 else 4))
      + (if x then (if f (x, false) then (0:ℚ) else 4) else (if f (x, false) then 5 else 0))
    = _
  rw [hfg (x, true) rfl, hfg (x, false) rfl]
  rfl

/-- Policies indexed by their two choices: `polW (ac, cc)` plays `ac` at
`o₁` and `cc` at `o₂`. -/
def polW : Bool × Bool → Bool → Bool := fun p o => if o then p.1 else p.2

/-- The value table, in one lemma. -/
theorem uDet_polW (ac cc x : Bool) :
    wit.uDet (polW (ac, cc)) x
      = (if x then (if ac then (1:ℚ) else 0) else (if ac then 0 else 4))
        + (if x then (if cc then (0:ℚ) else 4) else (if cc then 5 else 0)) := by
  cases x <;> rfl

/-- The weighted stock `{(δ_{w₁}, 1), (δ_{w₂}, t)}`. -/
def S2 (t : ℚ) (ht : 0 ≤ t) : Stock Bool Bool where
  pr k := FinPMFExtra.pure k
  wt k := if k then 1 else t
  wt_nonneg k := by
    cases k
    · simpa using ht
    · simp

@[simp] theorem S2_pr (t : ℚ) (ht : 0 ≤ t) (k : Bool) :
    (S2 t ht).pr k = FinPMFExtra.pure k := rfl

@[simp] theorem S2_wt_true (t : ℚ) (ht : 0 ≤ t) : (S2 t ht).wt true = 1 := rfl

@[simp] theorem S2_wt_false (t : ℚ) (ht : 0 ≤ t) : (S2 t ht).wt false = t := rfl

/-- Weighted regret over a two-member stock, elementwise (the crunching
route: two `le_wregret`/`wregret_le` bounds instead of kernel evaluation). -/
theorem wregret_S2_le {t : ℚ} {ht : 0 ≤ t} {ι : Type*} [Fintype ι] [Nonempty ι]
    {M : ι → Bool → ℚ} {i : ι} {c : ℚ}
    (h1 : regret M (FinPMFExtra.pure true) i ≤ c)
    (h2 : t * regret M (FinPMFExtra.pure false) i ≤ c) :
    wregret (S2 t ht) M i ≤ c := by
  apply wregret_le
  intro k
  cases k
  · simpa using h2
  · simpa using h1

theorem le_wregret_S2_true {t : ℚ} {ht : 0 ≤ t} {ι : Type*} [Fintype ι]
    [Nonempty ι] {M : ι → Bool → ℚ} {i : ι} :
    regret M (FinPMFExtra.pure true) i ≤ wregret (S2 t ht) M i := by
  have := le_wregret (S2 t ht) M i true
  simpa using this

theorem le_wregret_S2_false {t : ℚ} {ht : 0 ≤ t} {ι : Type*} [Fintype ι]
    [Nonempty ι] {M : ι → Bool → ℚ} {i : ι} :
    t * regret M (FinPMFExtra.pure false) i ≤ wregret (S2 t ht) M i := by
  have := le_wregret (S2 t ht) M i false
  simpa using this

/-! #### Global-menu values (the four policies, two Dirac measures)

Per-measure expectations of the policy menu `polMenu wit polW`:
`w₁`: `(a,c) ↦ 1, (a,d) ↦ 5, (b,c) ↦ 0, (b,d) ↦ 4` (benchmark 5 at `(a,d)`);
`w₂`: `(a,c) ↦ 5, (a,d) ↦ 0, (b,c) ↦ 9, (b,d) ↦ 4` (benchmark 9 at `(b,c)`). -/

theorem exp_polMenu (w ac cc : Bool) :
    (FinPMFExtra.pure w).exp (polMenu wit polW (ac, cc))
      = (if w then (if ac then (1:ℚ) else 0) else (if ac then 0 else 4))
        + (if w then (if cc then (0:ℚ) else 4) else (if cc then 5 else 0)) := by
  rw [show polMenu wit polW (ac, cc) = wit.uDet (polW (ac, cc)) from rfl,
    FinPMFExtra.exp_pure]
  exact uDet_polW ac cc w

theorem best_polMenu_true :
    best (polMenu wit polW) (FinPMFExtra.pure true) = 5 := by
  refine le_antisymm (best_le fun j => ?_) ?_
  · obtain ⟨ac, cc⟩ := j
    rw [exp_polMenu]
    cases ac <;> cases cc <;> norm_num
  · refine le_trans (le_of_eq ?_) (exp_le_best _ _ ((true, false) : Bool × Bool))
    rw [exp_polMenu]
    norm_num

theorem best_polMenu_false :
    best (polMenu wit polW) (FinPMFExtra.pure false) = 9 := by
  refine le_antisymm (best_le fun j => ?_) ?_
  · obtain ⟨ac, cc⟩ := j
    rw [exp_polMenu]
    cases ac <;> cases cc <;> norm_num
  · refine le_trans (le_of_eq ?_) (exp_le_best _ _ ((false, true) : Bool × Bool))
    rw [exp_polMenu]
    norm_num

theorem regret_polMenu_true (ac cc : Bool) :
    regret (polMenu wit polW) (FinPMFExtra.pure true) (ac, cc)
      = 5 - ((if ac then (1:ℚ) else 0) + (if cc then (0:ℚ) else 4)) := by
  rw [regret, best_polMenu_true, exp_polMenu]
  norm_num

theorem regret_polMenu_false (ac cc : Bool) :
    regret (polMenu wit polW) (FinPMFExtra.pure false) (ac, cc)
      = 9 - ((if ac then (0:ℚ) else 4) + (if cc then (5:ℚ) else 0)) := by
  rw [regret, best_polMenu_false, exp_polMenu]
  norm_num

/-! #### The witness, `t = 1`: unique selection `(a,c)`, not ratifiable -/

/-- `(a,c)` is the global menu-MWER choice at weights `(1,1)`. -/
theorem witness_selected :
    IsMWERChoice (S2 1 (by norm_num)) (polMenu wit polW) (true, true) := by
  intro j
  have hbound : wregret (S2 1 (by norm_num)) (polMenu wit polW) (true, true) ≤ 4 := by
    apply wregret_S2_le
    · rw [regret_polMenu_true]
      norm_num
    · rw [regret_polMenu_false]
      norm_num
  obtain ⟨ac, cc⟩ := j
  cases ac <;> cases cc
  · -- (b, d): wregret ≥ 5 via measure w₂
    refine le_trans hbound (le_trans ?_ le_wregret_S2_false)
    rw [regret_polMenu_false]
    norm_num
  · -- (b, c): wregret ≥ 5 via measure w₁
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_polMenu_true]
    norm_num
  · -- (a, d): wregret ≥ 9 via measure w₂
    refine le_trans hbound (le_trans ?_ le_wregret_S2_false)
    rw [regret_polMenu_false]
    norm_num
  · -- (a, c) itself
    exact le_refl _

/-- The selection is UNIQUE: every global MWER choice is `(a,c)`. -/
theorem witness_selected_unique (p : Bool × Bool)
    (hp : IsMWERChoice (S2 1 (by norm_num)) (polMenu wit polW) p) :
    p = (true, true) := by
  have hac : wregret (S2 1 (by norm_num)) (polMenu wit polW) (true, true) ≤ 4 := by
    apply wregret_S2_le
    · rw [regret_polMenu_true]; norm_num
    · rw [regret_polMenu_false]; norm_num
  obtain ⟨ac, cc⟩ := p
  cases ac <;> cases cc
  · exfalso
    have h5 : (5:ℚ) ≤ wregret (S2 1 (by norm_num)) (polMenu wit polW) (false, false) := by
      refine le_trans ?_ le_wregret_S2_false
      rw [regret_polMenu_false]; norm_num
    have := hp (true, true)
    linarith
  · exfalso
    have h5 : (5:ℚ) ≤ wregret (S2 1 (by norm_num)) (polMenu wit polW) (false, true) := by
      refine le_trans ?_ le_wregret_S2_true
      rw [regret_polMenu_true]; norm_num
    have := hp (true, true)
    linarith
  · exfalso
    have h9 : (9:ℚ) ≤ wregret (S2 1 (by norm_num)) (polMenu wit polW) (true, false) := by
      refine le_trans ?_ le_wregret_S2_false
      rw [regret_polMenu_false]; norm_num
    have := hp (true, true)
    linarith
  · rfl

/-! #### Fiber machinery for the witness -/

theorem subst_polW_o1 (ac cc d : Bool) :
    Scenario.subst (polW (ac, cc)) true d = polW (d, cc) := by
  funext i
  cases i <;> simp [Scenario.subst, polW]

theorem subst_polW_o2 (ac cc d : Bool) :
    Scenario.subst (polW (ac, cc)) false d = polW (ac, d) := by
  funext i
  cases i <;> simp [Scenario.subst, polW]

theorem exp_fiber_o1 (w ac cc d : Bool) :
    (FinPMFExtra.pure w).exp (wit.fiberMenuDet (polW (ac, cc)) true d)
      = (if w then (if d then (1:ℚ) else 0) else (if d then 0 else 4))
        + (if w then (if cc then (0:ℚ) else 4) else (if cc then 5 else 0)) := by
  show (FinPMFExtra.pure w).exp (wit.uDet (Scenario.subst (polW (ac, cc)) true d)) = _
  rw [subst_polW_o1, FinPMFExtra.exp_pure]
  exact uDet_polW d cc w

theorem exp_fiber_o2 (w ac cc d : Bool) :
    (FinPMFExtra.pure w).exp (wit.fiberMenuDet (polW (ac, cc)) false d)
      = (if w then (if ac then (1:ℚ) else 0) else (if ac then 0 else 4))
        + (if w then (if d then (0:ℚ) else 4) else (if d then 5 else 0)) := by
  show (FinPMFExtra.pure w).exp (wit.uDet (Scenario.subst (polW (ac, cc)) false d)) = _
  rw [subst_polW_o2, FinPMFExtra.exp_pure]
  exact uDet_polW ac d w

/-- **The witness** (paper §2.6, the FALSE preregistered sub-claim's
certificate): at weights `(1,1)`, the unique global menu-MWER optimum
`(a,c)` is NOT ratifiable — the fiber deviation `b` at `o₁` strictly beats
the incumbent in posterior fiber weighted regret. With `witness_selected` +
`witness_selected_unique`: `S_Π ∩ Rat_Π = ∅`. -/
theorem witness_not_ratifiable :
    ¬ Ratifiable (S2 1 (by norm_num)) wit (polW (true, true)) := by
  rw [ratifiable_iff_exante wit_local]
  intro H
  -- gate at o₁: P̄ > 0 (weight-1 member, trigger contains w₁)
  have hmem : (true : Bool) ∈ wit.trigger true :=
    (wit.mem_trigger true true).mpr ⟨(true, true), rfl, rfl⟩
  have hgate : 0 < (S2 1 (by norm_num)).pbar (wit.trigger true) := by
    have hle := Stock.le_pbar (S2 1 (by norm_num)) (wit.trigger true) true
    have hmass : Event.mass (FinPMFExtra.pure true) (wit.trigger true) = 1 := by
      rw [FinPMFExtra.mass_pure]
      simp [hmem]
    rw [S2_wt_true, S2_pr, hmass, one_mul] at hle
    linarith
  have hchoice := H true hgate
  -- incumbent's fiber index is `polW (true,true) true = true`
  have hcomp := hchoice false
  -- fiber regrets at o₁ through (a,c): incumbent (d = a): (0, 4); deviation
  -- b: (1, 0) — benchmark per measure: w₁ = 1 (at a), w₂ = 9 (at b).
  have hbest1 : best (wit.fiberMenuDet (polW (true, true)) true)
      (FinPMFExtra.pure true) = 1 := by
    refine le_antisymm (best_le fun d => ?_) ?_
    · rw [exp_fiber_o1]
      cases d <;> norm_num
    · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
      rw [exp_fiber_o1]
      norm_num
  have hbest2 : best (wit.fiberMenuDet (polW (true, true)) true)
      (FinPMFExtra.pure false) = 9 := by
    refine le_antisymm (best_le fun d => ?_) ?_
    · rw [exp_fiber_o1]
      cases d <;> norm_num
    · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
      rw [exp_fiber_o1]
      norm_num
  -- wregret(incumbent true) ≥ 4 (measure w₂); wregret(deviation false) ≤ 1
  have hinc : (4:ℚ) ≤ wregret (S2 1 (by norm_num))
      (wit.fiberMenuDet (polW (true, true)) true) true := by
    refine le_trans ?_ le_wregret_S2_false
    rw [regret, hbest2, exp_fiber_o1]
    norm_num
  have hdev : wregret (S2 1 (by norm_num))
      (wit.fiberMenuDet (polW (true, true)) true) false ≤ 1 := by
    apply wregret_S2_le
    · rw [regret, hbest1, exp_fiber_o1]
      norm_num
    · rw [regret, hbest2, exp_fiber_o1]
      norm_num
  -- hcomp : wregret(incumbent) ≤ wregret(deviation): 4 ≤ 1, absurd
  have : (4:ℚ) ≤ 1 := le_trans hinc (le_trans hcomp hdev)
  norm_num at this

/-- `S_Π ∩ Rat_Π = ∅`, packaged: every global MWER choice at the witness
stock fails ratifiability. -/
theorem witness_selection_ratification_disjoint (p : Bool × Bool)
    (hp : IsMWERChoice (S2 1 (by norm_num)) (polMenu wit polW) p) :
    ¬ Ratifiable (S2 1 (by norm_num)) wit (polW p) := by
  rw [witness_selected_unique p hp]
  exact witness_not_ratifiable

/-- **The converse gap** (relations row R8(ii), same instance): `(b,c)` is
coordinatewise stable (ratifiable) yet globally suboptimal — both directions
of the global/local gap in one exact-ℚ witness. -/
theorem witness_converse_gap :
    Ratifiable (S2 1 (by norm_num)) wit (polW (false, true))
    ∧ ¬ IsMWERChoice (S2 1 (by norm_num)) (polMenu wit polW) (false, true) := by
  constructor
  · rw [ratifiable_iff_exante wit_local]
    intro o _
    intro d
    cases o
    · -- o₂ fiber through (b,c): incumbent c beats d
      -- values: c ↦ (0,9), d ↦ (4,4); benchmarks: w₁ = 4 (at d), w₂ = 9 (at c)
      have hbest1 : best (wit.fiberMenuDet (polW (false, true)) false)
          (FinPMFExtra.pure true) = 4 := by
        refine le_antisymm (best_le fun e => ?_) ?_
        · rw [exp_fiber_o2]
          cases e <;> norm_num
        · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
          rw [exp_fiber_o2]
          norm_num
      have hbest2 : best (wit.fiberMenuDet (polW (false, true)) false)
          (FinPMFExtra.pure false) = 9 := by
        refine le_antisymm (best_le fun e => ?_) ?_
        · rw [exp_fiber_o2]
          cases e <;> norm_num
        · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
          rw [exp_fiber_o2]
          norm_num
      have hinc : wregret (S2 1 (by norm_num))
          (wit.fiberMenuDet (polW (false, true)) false) true ≤ 4 := by
        apply wregret_S2_le
        · rw [regret, hbest1, exp_fiber_o2]
          norm_num
        · rw [regret, hbest2, exp_fiber_o2]
          norm_num
      cases d
      · -- deviation d: wregret ≥ 5 (measure w₂: 9 − 4 = 5)
        refine le_trans hinc (le_trans ?_ le_wregret_S2_false)
        rw [regret, hbest2, exp_fiber_o2]
        norm_num
      · exact le_refl _
    · -- o₁ fiber through (b,c): incumbent b beats a
      -- values: a ↦ (1,5), b ↦ (0,9); benchmarks: w₁ = 1 (at a), w₂ = 9 (at b)
      have hbest1 : best (wit.fiberMenuDet (polW (false, true)) true)
          (FinPMFExtra.pure true) = 1 := by
        refine le_antisymm (best_le fun e => ?_) ?_
        · rw [exp_fiber_o1]
          cases e <;> norm_num
        · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
          rw [exp_fiber_o1]
          norm_num
      have hbest2 : best (wit.fiberMenuDet (polW (false, true)) true)
          (FinPMFExtra.pure false) = 9 := by
        refine le_antisymm (best_le fun e => ?_) ?_
        · rw [exp_fiber_o1]
          cases e <;> norm_num
        · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
          rw [exp_fiber_o1]
          norm_num
      have hinc : wregret (S2 1 (by norm_num))
          (wit.fiberMenuDet (polW (false, true)) true) false ≤ 1 := by
        apply wregret_S2_le
        · rw [regret, hbest1, exp_fiber_o1]
          norm_num
        · rw [regret, hbest2, exp_fiber_o1]
          norm_num
      cases d
      · exact le_refl _
      · -- deviation a: wregret ≥ 4 (measure w₂: 9 − 5 = 4)
        refine le_trans hinc (le_trans ?_ le_wregret_S2_false)
        rw [regret, hbest2, exp_fiber_o1]
        norm_num
  · intro H
    have h1 := H (true, true)
    have h5 : (5:ℚ) ≤ wregret (S2 1 (by norm_num)) (polMenu wit polW) (false, true) := by
      refine le_trans ?_ le_wregret_S2_true
      rw [regret_polMenu_true]
      norm_num
    have h4 : wregret (S2 1 (by norm_num)) (polMenu wit polW) (true, true) ≤ 4 := by
      apply wregret_S2_le
      · rw [regret_polMenu_true]; norm_num
      · rw [regret_polMenu_false]; norm_num
    linarith

/-! #### The singleton-limit control (`t = 1/100`): stability restores -/

/-- At `t = 1/100` the unique global optimum moves to `(a,d)` (measure 1's
EU-optimum) … -/
theorem control_selected :
    IsMWERChoice (S2 (1/100) (by norm_num)) (polMenu wit polW) (true, false) := by
  intro j
  have hbound : wregret (S2 (1/100) (by norm_num)) (polMenu wit polW) (true, false)
      ≤ 9/100 := by
    apply wregret_S2_le
    · rw [regret_polMenu_true]
      norm_num
    · rw [regret_polMenu_false]
      norm_num
  obtain ⟨ac, cc⟩ := j
  cases ac <;> cases cc
  · -- (b, d): wregret ≥ 1 via w₁
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_polMenu_true]
    norm_num
  · -- (b, c): wregret ≥ 5 via w₁
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_polMenu_true]
    norm_num
  · -- (a, d) itself
    exact le_refl _
  · -- (a, c): wregret ≥ 4 via w₁
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_polMenu_true]
    norm_num

/-- … and it IS ratifiable: the fence's own control — as the second weight
vanishes, selection and stability re-compose (Theorem 2's precise collapse
as the boundary case). -/
theorem control_ratifiable :
    Ratifiable (S2 (1/100) (by norm_num)) wit (polW (true, false)) := by
  rw [ratifiable_iff_exante wit_local]
  intro o _
  intro d
  cases o
  · -- o₂ fiber through (a,d): incumbent d; values c ↦ (1,5), d ↦ (5,0);
    -- benchmarks: w₁ = 5 (at d), w₂ = 5 (at c)
    have hbest1 : best (wit.fiberMenuDet (polW (true, false)) false)
        (FinPMFExtra.pure true) = 5 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o2]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
        rw [exp_fiber_o2]
        norm_num
    have hbest2 : best (wit.fiberMenuDet (polW (true, false)) false)
        (FinPMFExtra.pure false) = 5 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o2]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
        rw [exp_fiber_o2]
        norm_num
    have hinc : wregret (S2 (1/100) (by norm_num))
        (wit.fiberMenuDet (polW (true, false)) false) false ≤ 5/100 := by
      apply wregret_S2_le
      · rw [regret, hbest1, exp_fiber_o2]
        norm_num
      · rw [regret, hbest2, exp_fiber_o2]
        norm_num
    cases d
    · exact le_refl _
    · -- deviation c: wregret ≥ 4 via w₁ (5 − 1 = 4)
      refine le_trans hinc (le_trans ?_ le_wregret_S2_true)
      rw [regret, hbest1, exp_fiber_o2]
      norm_num
  · -- o₁ fiber through (a,d): incumbent a; values a ↦ (5,0), b ↦ (4,4);
    -- benchmarks: w₁ = 5 (at a), w₂ = 4 (at b)
    have hbest1 : best (wit.fiberMenuDet (polW (true, false)) true)
        (FinPMFExtra.pure true) = 5 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o1]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
        rw [exp_fiber_o1]
        norm_num
    have hbest2 : best (wit.fiberMenuDet (polW (true, false)) true)
        (FinPMFExtra.pure false) = 4 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o1]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
        rw [exp_fiber_o1]
        norm_num
    have hinc : wregret (S2 (1/100) (by norm_num))
        (wit.fiberMenuDet (polW (true, false)) true) true ≤ 4/100 := by
      apply wregret_S2_le
      · rw [regret, hbest1, exp_fiber_o1]
        norm_num
      · rw [regret, hbest2, exp_fiber_o1]
        norm_num
    cases d
    · -- deviation b: wregret ≥ 1 via w₁ (5 − 4 = 1)
      refine le_trans hinc (le_trans ?_ le_wregret_S2_true)
      rw [regret, hbest1, exp_fiber_o1]
      norm_num
    · exact le_refl _

/-! #### The parametric threshold (§3(5) as a limit statement — the α₂→0 sweep)

Prereg (committed before these proofs):
`hearth/maitria/prereg/W668-ALPHA2-SWEEP-PREREG.md`. Global weighted regrets
over `S2 t`: `W(a,c) = max(4, 4t)`, `W(a,d) = 9t`, `W(b,c) = 5`,
`W(b,d) = max(1, 5t)`; fiber weighted regrets (incumbent-independent, by
additivity of the witness utility): at `o₁` `a ↦ 4t, b ↦ 1`; at `o₂`
`c ↦ 4, d ↦ 5t`. The threshold is exact and rational: **t₀ = 1/9**.

* `stability_restored`: for all `0 ≤ t < 1/9`, every global menu-MWER
  choice is `(a,d)` and IS ratifiable — `S_Π ⊆ Rat_Π`, nonvacuously
  (`control_selected_param` holds through the closed endpoint).
* `sharpness`: for `1/9 < t < 1/4`, the unique choice `(b,d)` is NOT
  ratifiable — `S_Π ∩ Rat_Π = ∅` immediately above the threshold.
* `threshold_boundary` (`t = 1/9` itself): the tie — `(a,d)` and `(b,d)`
  both selected, only `(a,d)` ratifiable; the strict `<` in
  `stability_restored` is honest, not slack.
* `control_ratifiable_param`: ratifiability of `(a,d)` persists to exactly
  `t ≤ 1/4` — selection moves first as `t` grows.
* NON-monotonicity: instability windows `(1/9, 1/4)` (this section's
  `mutant_t_one_fifth`) and `(4/5, 5/4)` (the `t = 1` witness above); the
  landed `t = 1/100` control cells are the point `1/100 < 1/9` of the
  parametric statement (`example`s below). -/

section Threshold

variable {t : ℚ}

/-- Global regret values as equations (crunched once; `w₁` column). -/
theorem regret_ac_w1 :
    regret (polMenu wit polW) (FinPMFExtra.pure true)
      ((true, true) : Bool × Bool) = 4 := by
  rw [regret_polMenu_true]; norm_num

theorem regret_ad_w1 :
    regret (polMenu wit polW) (FinPMFExtra.pure true)
      ((true, false) : Bool × Bool) = 0 := by
  rw [regret_polMenu_true]; norm_num

theorem regret_bc_w1 :
    regret (polMenu wit polW) (FinPMFExtra.pure true)
      ((false, true) : Bool × Bool) = 5 := by
  rw [regret_polMenu_true]; norm_num

theorem regret_bd_w1 :
    regret (polMenu wit polW) (FinPMFExtra.pure true)
      ((false, false) : Bool × Bool) = 1 := by
  rw [regret_polMenu_true]; norm_num

/-- Global regret values as equations (`w₂` column). -/
theorem regret_ad_w2 :
    regret (polMenu wit polW) (FinPMFExtra.pure false)
      ((true, false) : Bool × Bool) = 9 := by
  rw [regret_polMenu_false]; norm_num

theorem regret_bd_w2 :
    regret (polMenu wit polW) (FinPMFExtra.pure false)
      ((false, false) : Bool × Bool) = 5 := by
  rw [regret_polMenu_false]; norm_num

/-- `W(a,d) ≤ 9t`, parametric. -/
theorem wregret_ad_le (ht : 0 ≤ t) :
    wregret (S2 t ht) (polMenu wit polW) (true, false) ≤ 9 * t := by
  apply wregret_S2_le
  · rw [regret_ad_w1]; linarith
  · rw [regret_ad_w2]; linarith

/-- `W(b,d) ≤ max(1, 5t)`, parametric. -/
theorem wregret_bd_le (ht : 0 ≤ t) :
    wregret (S2 t ht) (polMenu wit polW) (false, false) ≤ max 1 (5 * t) := by
  apply wregret_S2_le
  · rw [regret_bd_w1]; exact le_max_left _ _
  · rw [regret_bd_w2]
    exact le_trans (le_of_eq (mul_comm t 5)) (le_max_right _ _)

/-- Below and AT the threshold (`t ≤ 1/9`): `(a,d)` is a global menu-MWER
choice (nonvacuity of `stability_restored`; membership holds at the closed
endpoint). -/
theorem control_selected_param (ht : 0 ≤ t) (h9 : t ≤ 1/9) :
    IsMWERChoice (S2 t ht) (polMenu wit polW) (true, false) := by
  intro j
  have hbound := wregret_ad_le ht
  obtain ⟨ac, cc⟩ := j
  cases ac <;> cases cc
  · -- (b, d): wregret ≥ 1 via w₁, and 9t ≤ 1
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_bd_w1]; linarith
  · -- (b, c): wregret ≥ 5 via w₁
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_bc_w1]; linarith
  · -- (a, d) itself
    exact le_refl _
  · -- (a, c): wregret ≥ 4 via w₁
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_ac_w1]; linarith

/-- STRICTLY below the threshold (`t < 1/9`): the selection is UNIQUE —
every global MWER choice is `(a,d)`. (Fails at `t = 1/9`:
`threshold_boundary`.) -/
theorem control_selected_unique_param (ht : 0 ≤ t) (h9 : t < 1/9)
    (p : Bool × Bool) (hp : IsMWERChoice (S2 t ht) (polMenu wit polW) p) :
    p = (true, false) := by
  have hbound := wregret_ad_le ht
  obtain ⟨ac, cc⟩ := p
  cases ac <;> cases cc
  · exfalso -- (b, d): wregret ≥ 1 > 9t
    have h1 : (1:ℚ) ≤ wregret (S2 t ht) (polMenu wit polW) (false, false) :=
      le_trans (le_of_eq regret_bd_w1.symm) le_wregret_S2_true
    have hcmp := hp (true, false)
    linarith
  · exfalso -- (b, c): wregret ≥ 5 > 9t
    have h5 : (5:ℚ) ≤ wregret (S2 t ht) (polMenu wit polW) (false, true) :=
      le_trans (le_of_eq regret_bc_w1.symm) le_wregret_S2_true
    have hcmp := hp (true, false)
    linarith
  · rfl
  · exfalso -- (a, c): wregret ≥ 4 > 9t
    have h4 : (4:ℚ) ≤ wregret (S2 t ht) (polMenu wit polW) (true, true) :=
      le_trans (le_of_eq regret_ac_w1.symm) le_wregret_S2_true
    have hcmp := hp (true, false)
    linarith

/-- Ratifiability of `(a,d)` persists to exactly `t ≤ 1/4` (its own fiber
boundary: `4t ≤ 1` at `o₁` is the binding constraint) — strictly past the
selection threshold `1/9`, so selection is what moves first. -/
theorem control_ratifiable_param (ht : 0 ≤ t) (h4 : t ≤ 1/4) :
    Ratifiable (S2 t ht) wit (polW (true, false)) := by
  rw [ratifiable_iff_exante wit_local]
  intro o _
  intro d
  cases o
  · -- o₂ fiber through (a,d): incumbent d; values c ↦ (1,5), d ↦ (5,0);
    -- benchmarks: w₁ = 5 (at d), w₂ = 5 (at c)
    have hbest1 : best (wit.fiberMenuDet (polW (true, false)) false)
        (FinPMFExtra.pure true) = 5 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o2]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
        rw [exp_fiber_o2]
        norm_num
    have hbest2 : best (wit.fiberMenuDet (polW (true, false)) false)
        (FinPMFExtra.pure false) = 5 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o2]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
        rw [exp_fiber_o2]
        norm_num
    have hrd1 : regret (wit.fiberMenuDet (polW (true, false)) false)
        (FinPMFExtra.pure true) false = 0 := by
      rw [regret, hbest1, exp_fiber_o2]; norm_num
    have hrd2 : regret (wit.fiberMenuDet (polW (true, false)) false)
        (FinPMFExtra.pure false) false = 5 := by
      rw [regret, hbest2, exp_fiber_o2]; norm_num
    have hrc1 : regret (wit.fiberMenuDet (polW (true, false)) false)
        (FinPMFExtra.pure true) true = 4 := by
      rw [regret, hbest1, exp_fiber_o2]; norm_num
    have hinc : wregret (S2 t ht)
        (wit.fiberMenuDet (polW (true, false)) false) false ≤ 5 * t := by
      apply wregret_S2_le
      · rw [hrd1]; linarith
      · rw [hrd2]; linarith
    cases d
    · exact le_refl _
    · -- deviation c: wregret ≥ 4 via w₁, and 5t ≤ 5/4 ≤ 4
      refine le_trans hinc (le_trans ?_ le_wregret_S2_true)
      rw [hrc1]; linarith
  · -- o₁ fiber through (a,d): incumbent a; values a ↦ (5,0), b ↦ (4,4);
    -- benchmarks: w₁ = 5 (at a), w₂ = 4 (at b)
    have hbest1 : best (wit.fiberMenuDet (polW (true, false)) true)
        (FinPMFExtra.pure true) = 5 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o1]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
        rw [exp_fiber_o1]
        norm_num
    have hbest2 : best (wit.fiberMenuDet (polW (true, false)) true)
        (FinPMFExtra.pure false) = 4 := by
      refine le_antisymm (best_le fun e => ?_) ?_
      · rw [exp_fiber_o1]
        cases e <;> norm_num
      · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
        rw [exp_fiber_o1]
        norm_num
    have hra1 : regret (wit.fiberMenuDet (polW (true, false)) true)
        (FinPMFExtra.pure true) true = 0 := by
      rw [regret, hbest1, exp_fiber_o1]; norm_num
    have hra2 : regret (wit.fiberMenuDet (polW (true, false)) true)
        (FinPMFExtra.pure false) true = 4 := by
      rw [regret, hbest2, exp_fiber_o1]; norm_num
    have hrb1 : regret (wit.fiberMenuDet (polW (true, false)) true)
        (FinPMFExtra.pure true) false = 1 := by
      rw [regret, hbest1, exp_fiber_o1]; norm_num
    have hinc : wregret (S2 t ht)
        (wit.fiberMenuDet (polW (true, false)) true) true ≤ 4 * t := by
      apply wregret_S2_le
      · rw [hra1]; linarith
      · rw [hra2]; linarith
    cases d
    · -- deviation b: wregret ≥ 1 via w₁, and 4t ≤ 1
      refine le_trans hinc (le_trans ?_ le_wregret_S2_true)
      rw [hrb1]; linarith
    · exact le_refl _

/-- **THE PARAMETRIC-THRESHOLD THEOREM** (mechanization debt §3(5),
discharged as a limit statement): for ALL `0 ≤ t < 1/9`, every global
menu-MWER choice at the stock `S2 t` is ratifiable — `S_Π ⊆ Rat_Π`,
nonvacuously (`control_selected_param`). The threshold is exact
(`sharpness`); the landed `control_*` cells are its point `t = 1/100`. -/
theorem stability_restored (ht : 0 ≤ t) (h9 : t < 1/9)
    (p : Bool × Bool) (hp : IsMWERChoice (S2 t ht) (polMenu wit polW) p) :
    Ratifiable (S2 t ht) wit (polW p) := by
  rw [control_selected_unique_param ht h9 p hp]
  exact control_ratifiable_param ht (by linarith)

/-- The ∃-form headline (the debt's asked-for shape), with the threshold
PINNED to its exact rational value rather than merely asserted to exist. -/
theorem exists_threshold :
    ∃ t₀ : ℚ, 0 < t₀ ∧ ∀ (t : ℚ) (ht : 0 ≤ t), t < t₀ →
      ∀ p, IsMWERChoice (S2 t ht) (polMenu wit polW) p →
        Ratifiable (S2 t ht) wit (polW p) :=
  ⟨1/9, by norm_num, fun _ ht h9 p hp => stability_restored ht h9 p hp⟩

/-- On `[1/9, 4/5]`: `(b,d)` is a global menu-MWER choice. -/
theorem bd_selected_param (ht : 0 ≤ t) (h9 : 1/9 ≤ t) (h45 : t ≤ 4/5) :
    IsMWERChoice (S2 t ht) (polMenu wit polW) (false, false) := by
  intro j
  have hbound := wregret_bd_le ht
  obtain ⟨ac, cc⟩ := j
  cases ac <;> cases cc
  · -- (b, d) itself
    exact le_refl _
  · -- (b, c): wregret ≥ 5 via w₁, and max(1, 5t) ≤ 5
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_bc_w1]
    exact max_le (by norm_num) (by linarith)
  · -- (a, d): wregret ≥ 9t via w₂, and max(1, 5t) ≤ 9t
    refine le_trans hbound (le_trans ?_ le_wregret_S2_false)
    rw [regret_ad_w2]
    exact max_le (by linarith) (by linarith)
  · -- (a, c): wregret ≥ 4 via w₁, and max(1, 5t) ≤ 4
    refine le_trans hbound (le_trans ?_ le_wregret_S2_true)
    rw [regret_ac_w1]
    exact max_le (by norm_num) (by linarith)

/-- On the OPEN window `(1/9, 4/5)` the selection `(b,d)` is unique. -/
theorem bd_unique_param (ht : 0 ≤ t) (h9 : 1/9 < t) (h45 : t < 4/5)
    (p : Bool × Bool) (hp : IsMWERChoice (S2 t ht) (polMenu wit polW) p) :
    p = (false, false) := by
  have hbound := wregret_bd_le ht
  have hmax4 : max 1 (5 * t) < 4 := max_lt (by norm_num) (by linarith)
  obtain ⟨ac, cc⟩ := p
  cases ac <;> cases cc
  · rfl
  · exfalso -- (b, c): wregret ≥ 5 > max(1, 5t)
    have h5 : (5:ℚ) ≤ wregret (S2 t ht) (polMenu wit polW) (false, true) :=
      le_trans (le_of_eq regret_bc_w1.symm) le_wregret_S2_true
    have hcmp := hp (false, false)
    have hchain := le_trans hcmp hbound
    linarith
  · exfalso -- (a, d): wregret ≥ 9t > max(1, 5t)
    have h9t := le_wregret_S2_false (t := t) (ht := ht)
      (M := polMenu wit polW) (i := ((true, false) : Bool × Bool))
    rw [regret_ad_w2] at h9t
    have hmax9 : max 1 (5 * t) < 9 * t := max_lt (by linarith) (by linarith)
    have hcmp := hp (false, false)
    have hchain := le_trans hcmp hbound
    linarith
  · exfalso -- (a, c): wregret ≥ 4 > max(1, 5t)
    have h4 : (4:ℚ) ≤ wregret (S2 t ht) (polMenu wit polW) (true, true) :=
      le_trans (le_of_eq regret_ac_w1.symm) le_wregret_S2_true
    have hcmp := hp (false, false)
    have hchain := le_trans hcmp hbound
    linarith

/-- On `[0, 1/4)`: `(b,d)` is NOT ratifiable — its `o₁` coordinate `b` loses
the fiber to `a` (`4t < 1`). -/
theorem bd_not_ratifiable_param (ht : 0 ≤ t) (h4 : t < 1/4) :
    ¬ Ratifiable (S2 t ht) wit (polW (false, false)) := by
  rw [ratifiable_iff_exante wit_local]
  intro H
  -- gate at o₁: P̄ > 0 (weight-1 member, trigger contains w₁)
  have hmem : (true : Bool) ∈ wit.trigger true :=
    (wit.mem_trigger true true).mpr ⟨(true, true), rfl, rfl⟩
  have hgate : 0 < (S2 t ht).pbar (wit.trigger true) := by
    have hle := Stock.le_pbar (S2 t ht) (wit.trigger true) true
    have hmass : Event.mass (FinPMFExtra.pure true) (wit.trigger true) = 1 := by
      rw [FinPMFExtra.mass_pure]
      simp [hmem]
    rw [S2_wt_true, S2_pr, hmass, one_mul] at hle
    linarith
  have hchoice := H true hgate
  -- incumbent's fiber index is `polW (false,false) true = false`; deviation a
  have hcomp := hchoice true
  -- fiber o₁ through (b,d): values a ↦ (5,0), b ↦ (4,4);
  -- benchmarks: w₁ = 5 (at a), w₂ = 4 (at b)
  have hbest1 : best (wit.fiberMenuDet (polW (false, false)) true)
      (FinPMFExtra.pure true) = 5 := by
    refine le_antisymm (best_le fun e => ?_) ?_
    · rw [exp_fiber_o1]
      cases e <;> norm_num
    · refine le_trans (le_of_eq ?_) (exp_le_best _ _ true)
      rw [exp_fiber_o1]
      norm_num
  have hbest2 : best (wit.fiberMenuDet (polW (false, false)) true)
      (FinPMFExtra.pure false) = 4 := by
    refine le_antisymm (best_le fun e => ?_) ?_
    · rw [exp_fiber_o1]
      cases e <;> norm_num
    · refine le_trans (le_of_eq ?_) (exp_le_best _ _ false)
      rw [exp_fiber_o1]
      norm_num
  have hrb1 : regret (wit.fiberMenuDet (polW (false, false)) true)
      (FinPMFExtra.pure true) false = 1 := by
    rw [regret, hbest1, exp_fiber_o1]; norm_num
  have hra1 : regret (wit.fiberMenuDet (polW (false, false)) true)
      (FinPMFExtra.pure true) true = 0 := by
    rw [regret, hbest1, exp_fiber_o1]; norm_num
  have hra2 : regret (wit.fiberMenuDet (polW (false, false)) true)
      (FinPMFExtra.pure false) true = 4 := by
    rw [regret, hbest2, exp_fiber_o1]; norm_num
  have hinc : (1:ℚ) ≤ wregret (S2 t ht)
      (wit.fiberMenuDet (polW (false, false)) true) false :=
    le_trans (le_of_eq hrb1.symm) le_wregret_S2_true
  have hdev : wregret (S2 t ht)
      (wit.fiberMenuDet (polW (false, false)) true) true ≤ 4 * t := by
    apply wregret_S2_le
    · rw [hra1]; linarith
    · rw [hra2]; linarith
  -- hcomp : wregret(incumbent b) ≤ wregret(deviation a): 1 ≤ 4t, absurd
  have : (1:ℚ) ≤ 4 * t := le_trans hinc (le_trans hcomp hdev)
  linarith

/-- **SHARPNESS**: immediately above the threshold — on `(1/9, 1/4)` — every
global MWER choice is `(b,d)` and it is NOT ratifiable:
`S_Π ∩ Rat_Π = ∅`. So `t₀ = 1/9` is exact, and (with the `t = 1` witness)
stability restoration is NON-monotone in `t`. -/
theorem sharpness (ht : 0 ≤ t) (h9 : 1/9 < t) (h4 : t < 1/4)
    (p : Bool × Bool) (hp : IsMWERChoice (S2 t ht) (polMenu wit polW) p) :
    ¬ Ratifiable (S2 t ht) wit (polW p) := by
  rw [bd_unique_param ht h9 (by linarith) p hp]
  exact bd_not_ratifiable_param ht h4

/-- The boundary `t = 1/9` itself: the selection TIE — `(a,d)` and `(b,d)`
both selected, `(a,d)` ratifiable, `(b,d)` not. `S ∩ Rat ≠ ∅` but
`S ⊄ Rat`: the strict inequality in `stability_restored` is honest. -/
theorem threshold_boundary :
    IsMWERChoice (S2 (1/9) (by norm_num)) (polMenu wit polW) (true, false)
    ∧ Ratifiable (S2 (1/9) (by norm_num)) wit (polW (true, false))
    ∧ IsMWERChoice (S2 (1/9) (by norm_num)) (polMenu wit polW) (false, false)
    ∧ ¬ Ratifiable (S2 (1/9) (by norm_num)) wit (polW (false, false)) :=
  ⟨control_selected_param (by norm_num) (le_refl _),
   control_ratifiable_param (by norm_num) (by norm_num),
   bd_selected_param (by norm_num) (le_refl _) (by norm_num),
   bd_not_ratifiable_param (by norm_num) (by norm_num)⟩

/-- The wrong-threshold mutant, concrete: at `t = 1/5 ∈ (1/9, 1/4)` the
unique selection `(b,d)` is not ratifiable — `S ∩ Rat = ∅`. Kills the
mutant claims "t₀ = 1/4" and "stability is monotone below the witness". -/
theorem mutant_t_one_fifth :
    IsMWERChoice (S2 (1/5) (by norm_num)) (polMenu wit polW) (false, false)
    ∧ ∀ p, IsMWERChoice (S2 (1/5) (by norm_num)) (polMenu wit polW) p →
        ¬ Ratifiable (S2 (1/5) (by norm_num)) wit (polW p) :=
  ⟨bd_selected_param (by norm_num) (by norm_num) (by norm_num),
   fun p hp => sharpness (by norm_num) (by norm_num) (by norm_num) p hp⟩

/-- Consistency pins: the landed `t = 1/100` control cells are the point
`1/100 < 1/9` of the parametric theorems (the spot-checks are subsumed). -/
example : IsMWERChoice (S2 (1/100) (by norm_num)) (polMenu wit polW) (true, false) :=
  control_selected_param (by norm_num) (by norm_num)

example : Ratifiable (S2 (1/100) (by norm_num)) wit (polW (true, false)) :=
  control_ratifiable_param (by norm_num) (by norm_num)

end Threshold

end SelWitness

end BernsteinLogic.IDT
