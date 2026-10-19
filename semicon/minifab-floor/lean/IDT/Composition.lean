/-
W668 M2 — IDT mechanization: the composition machinery (mechanization debt §3(1)).

Source of truth: `hearth/maitria/w666-receipts/W668-THEOREMS.md` §§0–1 —
Lemmas 1.1–1.3 and Theorem 1 over finite weighted stocks, exact-ℚ, atop the
frozen `IDT.Basic` scaffold (M1, @ f324486a7). The kernel-checked HL/MWER
chart in the SU atlas is qtsl `SuFunctor/Universe/HausdorffKL.lean` (`hklW`,
weights `α = e^{−t}`) — cited, never re-derived; this file mechanizes the
decision layer.

Statement-fidelity notes (M2-FINDINGS.md carries the fuller ledger):

* `Stock.update` is UNNORMALIZED (Basic's design): weights rescale by
  `Pr_k(E)` with no division by `P̄(E)`. Consequently the slice lemma
  (paper §1.2) is an ON-THE-NOSE equality `wregret (S.update E) M i =
  wregret S M i` — the paper's display divides both sides by the positive
  normalizer `P̄(E)`, which moves no preference and no argmin. Likewise
  Lemma 1.1 and Theorem 1 hold UNCONDITIONALLY here: the paper's
  "provided `P̄ > 0`" hypotheses exist only to make HL's normalized
  posterior well-defined, not for the invariance content.
* Lemma 1.1 is stated as `Stock.WEq` (equal weights everywhere, equal
  measures at nonzero weight): the two junk conditionals at a zero-mass
  member may differ as pmfs, but HL's stocks-as-canonical-sets never carry
  such members, and `WEq.wregret_eq` shows `WEq` determines the whole MWER
  functional. (Prereg T1-M1's sup-grouping concern is absorbed by Basic's
  indexed-stock design; the grouping lemma itself is M3's.)
* Locality is the abstract hypothesis of paper §0 (`Scenario.Local`);
  Lemma 1.3 is proved for BOTH output conventions Basic provides —
  deterministic (`uDet`) and independent-draw mixed (`uMix`) — so its
  convention-insensitivity (the paper's face (ii)) is a pair of theorems,
  not a remark.
-/
import IDT.Basic
import Mathlib

namespace BernsteinLogic.IDT

open BernsteinLogic.Imprecise
open Finset

/-! ### Private helpers (sup' order algebra over ℚ; one fix point for pin drift) -/

private theorem sup'_congr_q {κ : Type*} {s : Finset κ} (H : s.Nonempty) {f g : κ → ℚ}
    (h : ∀ k ∈ s, f k = g k) : s.sup' H f = s.sup' H g :=
  le_antisymm (Finset.sup'_le _ _ fun k hk => (h k hk).le.trans (Finset.le_sup' g hk))
    (Finset.sup'_le _ _ fun k hk => (h k hk).ge.trans (Finset.le_sup' f hk))

private theorem sup'_sub_const {κ : Type*} {s : Finset κ} (H : s.Nonempty) (f : κ → ℚ)
    (c : ℚ) : s.sup' H f - c = s.sup' H fun k => f k - c := by
  apply le_antisymm
  · rw [sub_le_iff_le_add, Finset.sup'_le_iff]
    intro k hk
    have h1 := Finset.le_sup' (fun k => f k - c) hk
    linarith
  · rw [Finset.sup'_le_iff]
    intro k hk
    have h1 := Finset.le_sup' f hk
    linarith

private theorem mul_sup'_pos {κ : Type*} {s : Finset κ} (H : s.Nonempty) (f : κ → ℚ)
    {c : ℚ} (hc : 0 < c) : (s.sup' H fun k => c * f k) = c * s.sup' H f := by
  apply le_antisymm
  · rw [Finset.sup'_le_iff]
    intro k hk
    exact mul_le_mul_of_nonneg_left (Finset.le_sup' f hk) hc.le
  · obtain ⟨b, hb, hfb⟩ := Finset.exists_mem_eq_sup' H f
    rw [hfb]
    exact Finset.le_sup' (fun k => c * f k) hb

private theorem finPMF_ext {X : Type*} [Fintype X] {p q : FinPMF X}
    (h : ∀ x, p.prob x = q.prob x) : p = q := by
  obtain ⟨p₁, _, _⟩ := p
  obtain ⟨p₂, _, _⟩ := q
  obtain rfl : p₁ = p₂ := funext h
  rfl

variable {X : Type*} [Fintype X] [DecidableEq X]

/-! ### Event-layer lemmas (mass monotonicity, conditional mass/composition) -/

namespace Event

theorem mass_mono (p : FinPMF X) {E F : Finset X} (h : E ⊆ F) : mass p E ≤ mass p F :=
  Finset.sum_le_sum_of_subset_of_nonneg h fun x _ _ => p.nonneg x

theorem prob_eq_zero_of_mass_eq_zero {p : FinPMF X} {E : Finset X} (h : mass p E = 0)
    {x : X} (hx : x ∈ E) : p.prob x = 0 :=
  (Finset.sum_eq_zero_iff_of_nonneg fun y _ => p.nonneg y).mp h x hx

theorem mass_eq_zero_of_not_pos {p : FinPMF X} {E : Finset X} (h : ¬ 0 < mass p E) :
    mass p E = 0 :=
  le_antisymm (not_lt.mp h) (mass_nonneg p E)

/-- Conditional mass: `(Pr|E)(F) = Pr(F ∩ E)/Pr(E)` (positive-mass case). -/
theorem mass_condPMF {p : FinPMF X} {E : Finset X} (hE : 0 < mass p E) (F : Finset X) :
    mass (condPMF p E) F = mass p (F ∩ E) / mass p E := by
  simp only [mass, condPMF_of_pos hE]
  rw [Finset.sum_ite_mem]
  simp only [div_eq_mul_inv]
  rw [← Finset.sum_mul]

/-- Composition of conditionals at the measure level — the classical half of
paper Lemma 1.1 (`(Pr|E₁)|E₂ = Pr|(E₁∩E₂)`), positive-mass case. -/
theorem condPMF_condPMF {p : FinPMF X} {E₁ E₂ : Finset X} (h1 : 0 < mass p E₁)
    (h12 : 0 < mass p (E₁ ∩ E₂)) :
    condPMF (condPMF p E₁) E₂ = condPMF p (E₁ ∩ E₂) := by
  have h2 : 0 < mass (condPMF p E₁) E₂ := by
    rw [mass_condPMF h1, Finset.inter_comm E₂ E₁]
    exact div_pos h12 h1
  apply finPMF_ext
  intro x
  simp only [condPMF_of_pos h2, condPMF_of_pos h12, condPMF_of_pos h1,
    mass_condPMF h1, Finset.inter_comm E₂ E₁]
  by_cases hx2 : x ∈ E₂
  · by_cases hx1 : x ∈ E₁
    · have hx12 : x ∈ E₁ ∩ E₂ := Finset.mem_inter.mpr ⟨hx1, hx2⟩
      rw [if_pos hx2, if_pos hx1, if_pos hx12, div_div_div_cancel_right₀]
      exact h1.ne'
    · have hx12 : x ∉ E₁ ∩ E₂ := fun hc => hx1 (Finset.mem_inter.mp hc).1
      rw [if_pos hx2, if_neg hx1, if_neg hx12, zero_div]
  · have hx12 : x ∉ E₁ ∩ E₂ := fun hc => hx2 (Finset.mem_inter.mp hc).2
    rw [if_neg hx2, if_neg hx12]

end Event

/-- Difference of expectations of acts agreeing off `E` is carried by `E`. -/
theorem exp_sub_exp_off {p : FinPMF X} {E : Finset X} {f g : X → ℚ}
    (hfg : ∀ x ∉ E, f x = g x) :
    p.exp f - p.exp g = ∑ x ∈ E, p.prob x * (f x - g x) := by
  simp only [FinPMF.exp, ← Finset.sum_sub_distrib, ← mul_sub]
  exact (Finset.sum_subset (Finset.subset_univ E) fun x _ hx => by
    rw [hfg x hx, sub_self, mul_zero]).symm

/-! ### The stock normalizer `P̄` (HL's `max_k α_k·Pr_k(E)`) — used by the
ratifiability gate ("observed with positive probability") on the Selection
side; the composition theorems below need no positivity at all. -/

namespace Stock

variable {K : Type*} [Fintype K] [Nonempty K]

/-- `P̄(E) = max_k α_k·Pr_k(E)` (HL, MWR.tex:1036–1076). -/
def pbar (S : Stock X K) (E : Finset X) : ℚ :=
  univ.sup' univ_nonempty fun k => S.wt k * Event.mass (S.pr k) E

theorem le_pbar (S : Stock X K) (E : Finset X) (k : K) :
    S.wt k * Event.mass (S.pr k) E ≤ S.pbar E :=
  Finset.le_sup' (fun k => S.wt k * Event.mass (S.pr k) E) (mem_univ k)

theorem pbar_nonneg (S : Stock X K) (E : Finset X) : 0 ≤ S.pbar E := by
  obtain ⟨k⟩ := ‹Nonempty K›
  exact le_trans (mul_nonneg (S.wt_nonneg k) (Event.mass_nonneg _ _)) (S.le_pbar E k)

/-- On the precise stock, `P̄(E)` is just `Pr(E)`. -/
theorem pbar_precise (p : FinPMF X) (E : Finset X) :
    (Stock.precise p).pbar E = Event.mass p E := by
  unfold pbar
  simp [Stock.precise, Finset.sup'_const]

/-! ### Weighted equivalence of stocks -/

/-- Weighted equivalence: equal weights everywhere, equal measures wherever
the weight is nonzero. This is exactly what HL's canonical stocks-as-sets
identify (they carry no zero-weight members); `WEq.wregret_eq` shows it
determines the MWER functional on every menu. -/
def WEq (S T : Stock X K) : Prop :=
  (∀ k, S.wt k = T.wt k) ∧ ∀ k, S.wt k ≠ 0 → S.pr k = T.pr k

namespace WEq

protected theorem refl (S : Stock X K) : WEq S S := ⟨fun _ => rfl, fun _ _ => rfl⟩

protected theorem symm {S T : Stock X K} (h : WEq S T) : WEq T S :=
  ⟨fun k => (h.1 k).symm, fun k hk => (h.2 k (by rw [h.1 k]; exact hk)).symm⟩

protected theorem trans {S T U : Stock X K} (h1 : WEq S T) (h2 : WEq T U) : WEq S U :=
  ⟨fun k => (h1.1 k).trans (h2.1 k), fun k hk =>
    (h1.2 k hk).trans (h2.2 k (by rw [← h1.1 k]; exact hk))⟩

/-- Weighted-equivalent stocks have identical MWER objectives on every menu. -/
theorem wregret_eq {S T : Stock X K} (h : WEq S T) {ι : Type*} [Fintype ι]
    [Nonempty ι] (M : ι → X → ℚ) (i : ι) : wregret S M i = wregret T M i := by
  unfold wregret
  apply sup'_congr_q univ_nonempty
  intro k _
  by_cases hw : S.wt k = 0
  · rw [← h.1 k, hw, zero_mul, zero_mul]
  · rw [h.1 k, h.2 k hw]

theorem pbar_eq {S T : Stock X K} (h : WEq S T) (E : Finset X) :
    S.pbar E = T.pbar E := by
  unfold pbar
  apply sup'_congr_q univ_nonempty
  intro k _
  by_cases hw : S.wt k = 0
  · rw [← h.1 k, hw, zero_mul, zero_mul]
  · rw [h.1 k, h.2 k hw]

/-- Updating respects weighted equivalence. -/
protected theorem update {S T : Stock X K} (h : WEq S T) (E : Finset X) :
    WEq (S.update E) (T.update E) := by
  constructor
  · intro k
    simp only [Stock.update_wt]
    by_cases hw : S.wt k = 0
    · rw [← h.1 k, hw, zero_mul, zero_mul]
    · rw [h.1 k, h.2 k hw]
  · intro k hk
    simp only [Stock.update_pr]
    simp only [Stock.update_wt] at hk
    have hw : S.wt k ≠ 0 := fun h0 => hk (by rw [h0, zero_mul])
    rw [h.2 k hw]

end WEq

/-! ### Lemma 1.1 — composition of likelihood updates -/

/-- **Lemma 1.1** (composition of likelihood updates — paper §1.1, [cited: HL
MWR.tex:1128–1164], here re-proved over the unnormalized presentation, and
UNCONDITIONALLY): running two updates equals one update on the intersection,
up to weighted equivalence — hence with identical MWER objectives on every
menu (`WEq.wregret_eq`). The paper's "provided the right side is defined"
guard exists only for HL's normalized posterior. -/
theorem update_update (S : Stock X K) (E₁ E₂ : Finset X) :
    WEq ((S.update E₁).update E₂) (S.update (E₁ ∩ E₂)) := by
  constructor
  · intro k
    simp only [Stock.update_wt, Stock.update_pr]
    by_cases hk : 0 < Event.mass (S.pr k) E₁
    · rw [Event.mass_condPMF hk E₂, Finset.inter_comm E₂ E₁,
        mul_assoc, ← mul_div_assoc, mul_div_cancel_left₀ _ hk.ne']
    · have hk0 : Event.mass (S.pr k) E₁ = 0 := Event.mass_eq_zero_of_not_pos hk
      have h12 : Event.mass (S.pr k) (E₁ ∩ E₂) = 0 :=
        le_antisymm (le_trans (Event.mass_mono _ inter_subset_left) hk0.le)
          (Event.mass_nonneg _ _)
      rw [Event.condPMF_of_zero hk, hk0, h12]
      ring
  · intro k hk
    simp only [Stock.update_pr]
    simp only [Stock.update_wt, Stock.update_pr] at hk
    have h1 : Event.mass (S.pr k) E₁ ≠ 0 := fun h0 => hk (by rw [h0, mul_zero, zero_mul])
    have h1p : 0 < Event.mass (S.pr k) E₁ :=
      lt_of_le_of_ne (Event.mass_nonneg _ _) (Ne.symm h1)
    have h2 : Event.mass (Event.condPMF (S.pr k) E₁) E₂ ≠ 0 := fun h0 =>
      hk (by rw [h0, mul_zero])
    have h12 : 0 < Event.mass (S.pr k) (E₁ ∩ E₂) := by
      rw [Event.mass_condPMF h1p, Finset.inter_comm E₂ E₁] at h2
      have hne : Event.mass (S.pr k) (E₁ ∩ E₂) ≠ 0 := fun h0 => h2 (by rw [h0, zero_div])
      exact lt_of_le_of_ne (Event.mass_nonneg _ _) (Ne.symm hne)
    exact Event.condPMF_condPMF h1p h12

/-- Commutativity of updates (paper 1.1's `(𝒫⁺|E₁)|E₂ = (𝒫⁺|E₂)|E₁` face). -/
theorem update_comm (S : Stock X K) (E₁ E₂ : Finset X) :
    WEq ((S.update E₁).update E₂) ((S.update E₂).update E₁) := by
  refine (update_update S E₁ E₂).trans (WEq.symm ?_)
  have h2 := update_update S E₂ E₁
  rwa [Finset.inter_comm E₂ E₁] at h2

end Stock

/-! ### Lemma 1.2 — the slice lemma (HL's MDC mechanism, displayed) -/

section Slice

variable {K : Type*} [Fintype K] [Nonempty K] {ι : Type*} [Fintype ι] [Nonempty ι]

/-- Sub-lemma A (paper §1.2's display): on a menu whose acts pairwise agree
off `E`, regret against `Pr` is `Pr(E)` times regret against `Pr|E`
(positive-mass case). -/
theorem regret_eq_mass_mul_regret_condPMF {p : FinPMF X} {E : Finset X}
    (hE : 0 < Event.mass p E) {M : ι → X → ℚ}
    (hoff : ∀ i j, ∀ x ∉ E, M i x = M j x) (i : ι) :
    regret M p i = Event.mass p E * regret M (Event.condPMF p E) i := by
  have key : ∀ j : ι, p.exp (M j) - p.exp (M i)
      = Event.mass p E * ((Event.condPMF p E).exp (M j)
          - (Event.condPMF p E).exp (M i)) := by
    intro j
    rw [exp_sub_exp_off (fun x hx => hoff j i x hx),
      exp_sub_exp_off (p := Event.condPMF p E) (fun x hx => hoff j i x hx),
      Finset.mul_sum]
    apply Finset.sum_congr rfl
    intro x hx
    simp only [Event.condPMF_of_pos hE, if_pos hx]
    rw [← mul_assoc, ← mul_div_assoc, mul_div_cancel_left₀ _ hE.ne']
  unfold regret best
  rw [sup'_sub_const, sup'_sub_const, sup'_congr_q univ_nonempty fun j _ => key j,
    mul_sup'_pos univ_nonempty _ hE]

/-- Sub-lemma B (prereg T1-M2, discharged): at a zero-mass member, regret on
an off-`E`-agreeing menu vanishes. -/
theorem regret_eq_zero_of_mass_eq_zero {p : FinPMF X} {E : Finset X}
    (hE : Event.mass p E = 0) {M : ι → X → ℚ}
    (hoff : ∀ i j, ∀ x ∉ E, M i x = M j x) (i : ι) :
    regret M p i = 0 := by
  have hexp : ∀ j : ι, p.exp (M j) = p.exp (M i) := by
    intro j
    have h1 := exp_sub_exp_off (p := p) (fun x hx => hoff j i x hx)
    have hz : ∑ x ∈ E, p.prob x * (M j x - M i x) = 0 :=
      Finset.sum_eq_zero fun x hx => by
        rw [Event.prob_eq_zero_of_mass_eq_zero hE hx, zero_mul]
    linarith
  have hbest : best M p = p.exp (M i) := best_eq_of M p i fun j => (hexp j).le
  rw [regret, hbest, sub_self]

/-- **Lemma 1.2** (the slice lemma — paper §1.2; the mechanism inside HL's MDC
axiom and `thm:update`, MWR.tex:3063–3163): on a menu of acts that pairwise
agree off `E`, the post-update MWER objective EQUALS the prior one — in
Basic's unnormalized presentation the paper's `1/P̄(E)` factor is absorbed, so
this is an equality of functionals, unconditionally; preferences and argmins
coincide (`slice_isMWERChoice_iff`). -/
theorem Stock.slice (S : Stock X K) (E : Finset X) {M : ι → X → ℚ}
    (hoff : ∀ i j, ∀ x ∉ E, M i x = M j x) (i : ι) :
    wregret (S.update E) M i = wregret S M i := by
  unfold wregret
  apply sup'_congr_q univ_nonempty
  intro k _
  simp only [Stock.update_wt, Stock.update_pr]
  by_cases hk : 0 < Event.mass (S.pr k) E
  · rw [regret_eq_mass_mul_regret_condPMF hk hoff i]
    ring
  · have hk0 : Event.mass (S.pr k) E = 0 := Event.mass_eq_zero_of_not_pos hk
    rw [Event.condPMF_of_zero hk, hk0, regret_eq_zero_of_mass_eq_zero hk0 hoff i]
    ring

/-- Preference/argmin form of the slice lemma: updating on `E` never moves an
MWER verdict on an off-`E`-agreeing menu. -/
theorem Stock.slice_isMWERChoice_iff (S : Stock X K) (E : Finset X) {M : ι → X → ℚ}
    (hoff : ∀ i j, ∀ x ∉ E, M i x = M j x) (i : ι) :
    IsMWERChoice (S.update E) M i ↔ IsMWERChoice S M i := by
  unfold IsMWERChoice
  simp only [Stock.slice S E hoff]

end Slice

/-! ### Lemma 1.3 — off-event constancy (the ONE new lemma), both conventions -/

namespace Scenario

variable {Mo I A : Type*}

/-- Substituting the incumbent's own output is the identity. -/
theorem subst_self [DecidableEq I] (π : I → A) (o : I) : subst π o (π o) = π := by
  funext i
  simp only [subst]
  split
  · next h => rw [h]
  · rfl

theorem substMix_self [DecidableEq I] [Fintype A] (π : I → FinPMF A) (o : I) :
    substMix π o (π o) = π := by
  funext i
  simp only [substMix]
  split
  · next h => rw [h]
  · rfl

variable [Fintype Mo] [DecidableEq I]

section Det

variable {S : Scenario X Mo I A}

/-- **Lemma 1.3, deterministic face** (off-event constancy — paper §1.3,
relations menu R7(c)): off the trigger `E_o`, a policy substituted at `o`
has the incumbent's utility — the EDT predicate manufactures exactly the
menu class (pairwise agreement off the conditioning event) that the slice
lemma consumes, with `E` the FNC trigger of the same observation. Off `E_o`
the input `o` is unrealized, so `π_{o→a}` and `π` agree on every realized
input; locality does the rest. -/
theorem uDet_subst_off_trigger (hLoc : S.Local) (π : I → A) (o : I) (a : A) {x : X}
    (hx : x ∉ S.trigger o) : S.uDet (subst π o a) x = S.uDet π x := by
  apply hLoc
  intro m hm
  have hne : S.inp m ≠ o := fun h => hx ((S.mem_trigger o x).mpr ⟨m, hm, h⟩)
  simp [subst, hne]

/-- Pairwise form of Lemma 1.3 (det), in the shape `Stock.slice` consumes. -/
theorem fiberMenuDet_agree_off (hLoc : S.Local) (π : I → A) (o : I) (a a' : A) {x : X}
    (hx : x ∉ S.trigger o) :
    S.fiberMenuDet π o a x = S.fiberMenuDet π o a' x := by
  unfold fiberMenuDet
  rw [uDet_subst_off_trigger hLoc π o a hx, uDet_subst_off_trigger hLoc π o a' hx]

/-- The incumbent's index in the fiber menu carries the incumbent's act. -/
theorem fiberMenuDet_self (π : I → A) (o : I) :
    S.fiberMenuDet π o (π o) = S.uDet π := by
  unfold fiberMenuDet
  rw [subst_self]

end Det

section Mix

variable [DecidableEq Mo] [Fintype A] [Nonempty A]
variable {S : Scenario X Mo I A}

/-- Locality transfers to the independent-draw mixed evaluation: `uMix` at
world `x` reads the mixed policy only at `x`'s own realized inputs — the
marginalization argument: outputs at other worlds' moments integrate out to
`1` by `FinPMF.total`. This is what makes Lemma 1.3's convention-
insensitivity (paper §1.3, face (ii)) a theorem for the mixed convention. -/
theorem uMix_congr_local (hLoc : S.Local) {π π' : I → FinPMF A} {x : X}
    (hagree : ∀ m : Mo, S.world m = x → π (S.inp m) = π' (S.inp m)) :
    S.uMix π x = S.uMix π' x := by
  have main : ∀ ρ : I → FinPMF A,
      (∀ m : Mo, S.world m = x → ρ (S.inp m) = π (S.inp m)) →
      S.uMix ρ x = ∑ g : {m : Mo // S.world m = x} → A,
        (∏ m : {m : Mo // S.world m = x}, (π (S.inp m.1)).prob (g m)) *
          S.payoff x fun m =>
            if hm : S.world m = x then g ⟨m, hm⟩ else Classical.arbitrary A := by
    intro ρ hρ
    set e := Equiv.piEquivPiSubtypeProd (fun m : Mo => S.world m = x)
      (fun _ : Mo => A) with he
    have hone : ∑ h : {m : Mo // ¬ S.world m = x} → A,
        ∏ m : {m : Mo // ¬ S.world m = x}, (ρ (S.inp m.1)).prob (h m) = 1 := by
      rw [← Fintype.piFinset_univ, ← Finset.prod_univ_sum]
      exact Finset.prod_eq_one fun m _ => (ρ (S.inp m.1)).total
    calc S.uMix ρ x
        = ∑ f : Mo → A, (∏ m, (ρ (S.inp m)).prob (f m)) * S.payoff x f := rfl
      _ = ∑ q : ({m : Mo // S.world m = x} → A) × ({m : Mo // ¬ S.world m = x} → A),
            (∏ m, (ρ (S.inp m)).prob (e.symm q m)) * S.payoff x (e.symm q) :=
          (Equiv.sum_comp e.symm _).symm
      _ = ∑ g : {m : Mo // S.world m = x} → A,
            ∑ h : {m : Mo // ¬ S.world m = x} → A,
            (∏ m, (ρ (S.inp m)).prob (e.symm (g, h) m)) * S.payoff x (e.symm (g, h)) :=
          Fintype.sum_prod_type _
      _ = _ := by
          apply Finset.sum_congr rfl
          intro g _
          have hsplit : ∀ h : {m : Mo // ¬ S.world m = x} → A,
              (∏ m : Mo, (ρ (S.inp m)).prob (e.symm (g, h) m))
                = (∏ m : {m : Mo // S.world m = x}, (π (S.inp m.1)).prob (g m)) *
                  ∏ m : {m : Mo // ¬ S.world m = x}, (ρ (S.inp m.1)).prob (h m) := by
            intro h
            rw [← Fintype.prod_subtype_mul_prod_subtype (fun m : Mo => S.world m = x)]
            congr 1
            · refine Finset.prod_congr rfl fun m _ => ?_
              rw [he]
              rw [Equiv.piEquivPiSubtypeProd_symm_apply, dif_pos m.2, hρ m.1 m.2]
            · refine Finset.prod_congr rfl fun m _ => ?_
              rw [he]
              rw [Equiv.piEquivPiSubtypeProd_symm_apply, dif_neg m.2]
          have hpay : ∀ h : {m : Mo // ¬ S.world m = x} → A,
              S.payoff x (e.symm (g, h))
                = S.payoff x fun m =>
                    if hm : S.world m = x then g ⟨m, hm⟩ else Classical.arbitrary A := by
            intro h
            apply hLoc
            intro m hm
            rw [he]
            rw [Equiv.piEquivPiSubtypeProd_symm_apply, dif_pos hm, dif_pos hm]
          calc ∑ h : {m : Mo // ¬ S.world m = x} → A,
                (∏ m, (ρ (S.inp m)).prob (e.symm (g, h) m)) * S.payoff x (e.symm (g, h))
              = ∑ h : {m : Mo // ¬ S.world m = x} → A,
                  ((∏ m : {m : Mo // S.world m = x}, (π (S.inp m.1)).prob (g m)) *
                    S.payoff x (fun m =>
                      if hm : S.world m = x then g ⟨m, hm⟩ else Classical.arbitrary A)) *
                    ∏ m : {m : Mo // ¬ S.world m = x}, (ρ (S.inp m.1)).prob (h m) := by
                refine Finset.sum_congr rfl fun h _ => ?_
                rw [hsplit h, hpay h]
                ring
            _ = _ := by rw [← Finset.mul_sum, hone, mul_one]
  rw [main π fun _ _ => rfl, main π' fun m hm => (hagree m hm).symm]

/-- **Lemma 1.3, mixed face**: off the trigger, a mixed policy substituted at
`o` has the incumbent's `uMix` utility. -/
theorem uMix_substMix_off_trigger (hLoc : S.Local) (π : I → FinPMF A) (o : I)
    (α : FinPMF A) {x : X} (hx : x ∉ S.trigger o) :
    S.uMix (substMix π o α) x = S.uMix π x := by
  apply uMix_congr_local hLoc
  intro m hm
  have hne : S.inp m ≠ o := fun h => hx ((S.mem_trigger o x).mpr ⟨m, hm, h⟩)
  simp [substMix, hne]

/-- Pairwise form of Lemma 1.3 (mix), in the shape `Stock.slice` consumes:
any two fiber-menu acts agree off the trigger. -/
theorem fiberMenuMix_agree_off (hLoc : S.Local) (π : I → FinPMF A) (o : I)
    {ι' : Type*} (devs : ι' → FinPMF A) (d d' : ι') {x : X}
    (hx : x ∉ S.trigger o) :
    S.fiberMenuMix π o devs d x = S.fiberMenuMix π o devs d' x := by
  unfold fiberMenuMix
  rw [uMix_substMix_off_trigger hLoc π o (devs d) hx,
    uMix_substMix_off_trigger hLoc π o (devs d') hx]

/-- The incumbent's own pmf indexes the incumbent's act in the mixed fiber
menu. -/
theorem fiberMenuMix_self (π : I → FinPMF A) (o : I) {ι' : Type*}
    (devs : ι' → FinPMF A) (d : ι') (hd : devs d = π o) :
    S.fiberMenuMix π o devs d = S.uMix π := by
  unfold fiberMenuMix
  rw [hd, substMix_self]

end Mix

end Scenario

/-! ### Theorem 1 — dynamic consistency under record persistence -/

namespace Stock

variable {K : Type*} [Fintype K] [Nonempty K]

/-- The running (iterated) likelihood update along a sequence of observed
events. -/
def iterUpdate (S : Stock X K) (E : ℕ → Finset X) : ℕ → Stock X K
  | 0 => S.update (E 0)
  | t + 1 => (iterUpdate S E t).update (E (t + 1))

/-- **Theorem 1(i)** (scheme invariance — paper §1.4): along a record-
persistent chain `E₀ ⊇ E₁ ⊇ ⋯`, the running update at stage `t` is
weighted-equivalent to the recompute-from-current-evidence update `𝒫⁺|E_t` —
unconditionally, in the unnormalized presentation (the paper's `P̄(E_T) > 0`
guard exists only for HL's normalized posterior). The chain hypothesis is
load-bearing: the paper's §1.4 erasure fence exhibits a preference-family
divergence without it (the explicit erasure witness is mechanization debt
§3(3), owned by M1). -/
theorem scheme_invariance (S : Stock X K) (E : ℕ → Finset X)
    (hchain : ∀ i, E (i + 1) ⊆ E i) :
    ∀ t : ℕ, WEq (iterUpdate S E t) (S.update (E t)) := by
  intro t
  induction t with
  | zero => exact WEq.refl _
  | succ n ih =>
    have hinter : E n ∩ E (n + 1) = E (n + 1) := Finset.inter_eq_right.mpr (hchain n)
    have step : WEq (iterUpdate S E (n + 1)) ((S.update (E n)).update (E (n + 1))) :=
      ih.update (E (n + 1))
    have h11 : WEq ((S.update (E n)).update (E (n + 1))) (S.update (E (n + 1))) := by
      have h := update_update S (E n) (E (n + 1))
      rwa [hinter] at h
    exact step.trans h11

end Stock

section Theorem1

set_option maxHeartbeats 1000000

variable {K : Type*} [Fintype K] [Nonempty K] {Mo I A : Type*}
variable [Fintype Mo] [DecidableEq I]

/-- **Theorem 1(ii)** (fiber-level ex-ante coherence, single-update form,
deterministic convention): the post-update MWER objective on any
deterministic fiber menu through any incumbent EQUALS the ex-ante objective —
updating moves credences, never a fiber verdict. Assembly: Lemma 1.3
manufactures the slice lemma's menu condition with `E` = the trigger of the
very same observation. -/
theorem fiber_coherence_det [Fintype A] [Nonempty A]
    {S : Scenario X Mo I A} (hLoc : S.Local) (T : Stock X K) (π : I → A) (o : I)
    (a : A) :
    wregret (T.update (S.trigger o)) (S.fiberMenuDet π o) a
      = wregret T (S.fiberMenuDet π o) a := by
  exact T.slice (S.trigger o) (M := S.fiberMenuDet π o)
    (fun d d' x hx => Scenario.fiberMenuDet_agree_off hLoc π o d d' hx) a

/-- **THEOREM 1** (paper §1.4, packaged, deterministic convention): along a
record-persistent chain of triggers with the current trigger
`E_t = E_{o_t}`, the RUNNING posterior MWER objective on the fiber menu
`F_{o_t,π}` equals the ex-ante objective — so no preference reversal occurs
anywhere along the chain (`theorem1_det_isMWERChoice_iff` is the
verdict-level form).

Fences (paper §1.4): fiber-level, not global — `IDT.Selection`'s §2.6
witness shows the global-menu analogue is FALSE at general stocks; the chain
hypothesis is load-bearing (erasure fence, M1's witness); triggers are
choice-independent by the `Scenario` construction. -/
theorem theorem1_det [Fintype A] [Nonempty A]
    {S : Scenario X Mo I A} (hLoc : S.Local) (T : Stock X K) (E : ℕ → Finset X)
    (hchain : ∀ i, E (i + 1) ⊆ E i) (t : ℕ) (π : I → A) (o : I)
    (htrig : E t = S.trigger o) (a : A) :
    wregret (T.iterUpdate E t) (S.fiberMenuDet π o) a
      = wregret T (S.fiberMenuDet π o) a := by
  rw [(T.scheme_invariance E hchain t).wregret_eq (S.fiberMenuDet π o) a, htrig]
  exact fiber_coherence_det hLoc T π o a

/-- Verdict form of Theorem 1 (det): the running posterior and ex-ante MWER
choice sets on the stage-`t` fiber menu coincide. -/
theorem theorem1_det_isMWERChoice_iff [Fintype A] [Nonempty A]
    {S : Scenario X Mo I A} (hLoc : S.Local) (T : Stock X K) (E : ℕ → Finset X)
    (hchain : ∀ i, E (i + 1) ⊆ E i) (t : ℕ) (π : I → A) (o : I)
    (htrig : E t = S.trigger o) (a : A) :
    IsMWERChoice (T.iterUpdate E t) (S.fiberMenuDet π o) a
      ↔ IsMWERChoice T (S.fiberMenuDet π o) a := by
  unfold IsMWERChoice
  simp only [theorem1_det hLoc T E hchain t π o htrig]

/-- Theorem 1(ii), mixed convention: same coherence for `fiberMenuMix` over
any finite family of deviation pmfs. -/
theorem fiber_coherence_mix [DecidableEq Mo] [Fintype A] [Nonempty A]
    {S : Scenario X Mo I A} (hLoc : S.Local) (T : Stock X K) (π : I → FinPMF A)
    (o : I) {ι' : Type*} [Fintype ι'] [Nonempty ι'] (devs : ι' → FinPMF A) (d : ι') :
    wregret (T.update (S.trigger o)) (S.fiberMenuMix π o devs) d
      = wregret T (S.fiberMenuMix π o devs) d := by
  exact T.slice (S.trigger o) (M := S.fiberMenuMix π o devs)
    (fun d d' x hx => Scenario.fiberMenuMix_agree_off hLoc π o devs d d' hx) d

/-- Theorem 1 packaged, mixed convention. -/
theorem theorem1_mix [DecidableEq Mo] [Fintype A] [Nonempty A]
    {S : Scenario X Mo I A} (hLoc : S.Local) (T : Stock X K) (E : ℕ → Finset X)
    (hchain : ∀ i, E (i + 1) ⊆ E i) (t : ℕ) (π : I → FinPMF A) (o : I)
    (htrig : E t = S.trigger o) {ι' : Type*} [Fintype ι'] [Nonempty ι']
    (devs : ι' → FinPMF A) (d : ι') :
    wregret (T.iterUpdate E t) (S.fiberMenuMix π o devs) d
      = wregret T (S.fiberMenuMix π o devs) d := by
  rw [(T.scheme_invariance E hchain t).wregret_eq (S.fiberMenuMix π o devs) d, htrig]
  exact fiber_coherence_mix hLoc T π o devs d

end Theorem1

end BernsteinLogic.IDT
