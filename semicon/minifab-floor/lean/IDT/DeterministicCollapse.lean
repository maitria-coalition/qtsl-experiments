/-
W676 Mini-Fab IDT-correctness — the deterministic point-mass collapse.

The carved-clock Mini-Fab closed system is DETERMINISTIC given a dispatch
policy (davidad's 2026-08-02 16:48 ruling: the demon keeps nothing), so the
belief state at the root-width choice locus (W668 post, Definition 7) is
precise at a point mass, and IDT selection collapses to makespan
minimization. This module proves that collapse against the landed W668
machinery (`isMWERChoice_precise_iff`, `FinPMFExtra.exp_pure`) and
instantiates it at a makespan-scored menu, plus the prereg M4 mutant
refutation (a two-policy concrete cell where the slower policy is provably
NOT an MWER choice).

Consumer: hearth/maitria/w6xx-receipts/W676-IDT-OPTIMALITY-FINDINGS.md.
Prereg: hearth/maitria/prereg/W676-idt-optimality-prereg.md (E6, M4).
Ownership: new file, disjoint from the W668 wave's per-sibling files.
-/
import IDT.Selection

namespace BernsteinLogic.IDT

open BernsteinLogic.Imprecise

section DeterministicCollapse

variable {X : Type*} [Fintype X] [DecidableEq X]
variable {ι : Type*} [Fintype ι] [Nonempty ι]

/-- **Point-mass collapse.** At a one-world (deterministic-dynamics) belief
state, MWER choice is plain utility maximization over the menu: the
imprecision machinery is inert exactly when the closed system keeps no
demonic width. -/
theorem isMWERChoice_pure_iff (x0 : X) (M : ι → X → ℚ) (i : ι) :
    IsMWERChoice (Stock.precise (FinPMFExtra.pure x0)) M i ↔
      ∀ j, M j x0 ≤ M i x0 := by
  rw [isMWERChoice_precise_iff]
  simp only [FinPMFExtra.exp_pure]

/-- **Makespan instantiation.** Utility = negated makespan at the single
world; IDT-selected at the root-width locus ⟺ makespan-minimal. This is
the sense in which "IDT-correct" for the synthesized Mini-Fab policy is
exactly "optimal throughput": with deterministic closed dynamics the
credal machinery adds nothing beyond the scheduling optimum. -/
theorem isMWERChoice_makespan_iff (x0 : X) (makespan : ι → ℚ) (i : ι) :
    IsMWERChoice (Stock.precise (FinPMFExtra.pure x0))
        (fun j _ => -makespan j) i ↔
      ∀ j, makespan i ≤ makespan j := by
  rw [isMWERChoice_pure_iff]
  constructor
  · intro h j
    have h1 : -makespan j ≤ -makespan i := h j
    linarith
  · intro h j
    have h1 : makespan i ≤ makespan j := h j
    show -makespan j ≤ -makespan i
    linarith

/-- **Zero-regret face.** At the point mass, weighted regret of a policy is
its makespan gap to the menu optimum — the bridge to the container-side
seat-regret (`SRegretLE`) reading: regret 0 at the optimum, and an
`SRegretLE` bound with ε = (makespan gap) elsewhere. -/
theorem wregret_makespan (x0 : X) (makespan : ι → ℚ) (i : ι) :
    wregret (Stock.precise (FinPMFExtra.pure x0)) (fun j _ => -makespan j) i =
      (Finset.univ.sup' Finset.univ_nonempty fun j => -makespan j) +
        makespan i := by
  rw [wregret_precise]
  unfold regret best
  simp only [FinPMFExtra.exp_pure]
  ring

/-! ### Prereg M4 concrete cell (two policies, makespans 5 and 7) -/

/-- Two-policy makespan table: policy 0 finishes in 5 rounds, policy 1
in 7. -/
private def mk2 : Fin 2 → ℚ := fun j => if j = 0 then 5 else 7

/-- The faster policy IS an MWER choice at the point mass. -/
example :
    IsMWERChoice (Stock.precise (FinPMFExtra.pure (0 : Fin 1)))
      (fun j (_ : Fin 1) => -mk2 j) 0 := by
  rw [isMWERChoice_makespan_iff]
  intro j
  fin_cases j <;> norm_num [mk2]

/-- M4 mutant refutation: the slower policy is NOT an MWER choice — the
flipped verdict is concretely false, not merely unproven. -/
example :
    ¬ IsMWERChoice (Stock.precise (FinPMFExtra.pure (0 : Fin 1)))
      (fun j (_ : Fin 1) => -mk2 j) 1 := by
  rw [isMWERChoice_makespan_iff]
  intro hall
  have h0 : mk2 1 ≤ mk2 0 := hall 0
  norm_num [mk2] at h0

end DeterministicCollapse

end BernsteinLogic.IDT
