/-
W676 Mini-Fab certified real-time floor — the machine-contiguity
strengthening: 255N + 371 → 255N + 375 for N ≥ 4 (and 255N + 373 for
N ≥ 3), kernel-checked.

Model: an abstract carved-clock schedule of the Mini-Fab's first five steps
(step 6 is WLOG-deleted for the arrivals objective, per the solver
encoding's documented reduction). The hypotheses are exactly the three
landed floor lemmas' premises plus machine capacity 2 at the diffusion
station, in machine-token-assignment form:

- per-lot precedence chains with one-round token dwell (Lemma A's premises);
- the per-schema meter law: each tick schema fires at most once per round,
  rendered as cross-lot disjointness of same-step tick rounds (Lemma B's
  premise — note it binds tick1 as well as tick5: one clock per schema);
- tick counts inside open service windows (Lemma C's demand structure);
- the two diffusion machines: each step-1/step-5 service holds a concrete
  machine token from start to finish; same-token services are disjoint.

The model is DELIBERATELY WEAKER than the net (no release meter, no
start/finish meters, no station-1/2 capacities, no w-guards), so every run
of the carved net projects onto an `MFSchedule` and the floor transfers a
fortiori.

Proof route (prereg R, hearth/maitria/prereg/W676-floor-contiguity-prereg.md):
the five-case machine-assignment tree of the findings §4 collapses — the
same premises FORCE the assignment. Entrant 3's step-1 service provably
overlaps both of the two earliest step-5 services, so (two machines,
pigeonhole) those two share a machine and the second entrant's first tick
is ≥ 628; entrant 4's step-1 service then overlaps the second and third
step-5 services, forcing t³ ≥ 885 against the tick5-meter budget t³ ≤ 884.
The attaining family π* realizes exactly this shape (double B→B handover,
slots at 371/628/885), so the bound is tight; with the solver's SAT
witnesses at 255N + 375 (N = 4..7) the floor certifies T_opt there
two-sidedly, and the N = 3 corollary certifies T_opt(3) = 1138.

Consumer: hearth/maitria/w6xx-receipts/W676-IDT-OPTIMALITY-FINDINGS.md §4
(Theorem D's [argued] grade lifts to kernel-checked at these statements).
Prereg: hearth/maitria/prereg/W676-floor-contiguity-prereg.md (E1–E3,
mutant loci M-A..M-D). Ownership: new file, disjoint from the W668/W676
sibling modules.
-/
import Mathlib

namespace BernsteinLogic.MiniFab

/-- Carved processing times of steps 1..5 (net constants T with step 6
dropped), verified at mtk main bytes (`minifab.rs`): T = [225,30,55,50,255]. -/
def procTime : Fin 5 → ℕ := ![225, 30, 55, 50, 255]

@[simp] lemma procTime_zero : procTime 0 = 225 := rfl
@[simp] lemma procTime_one : procTime 1 = 30 := rfl
@[simp] lemma procTime_two : procTime 2 = 55 := rfl
@[simp] lemma procTime_three : procTime 3 = 50 := rfl
@[simp] lemma procTime_four : procTime 4 = 255 := rfl

lemma procTime_pos (k : Fin 5) : 0 < procTime k := by fin_cases k <;> decide

/-- Abstract carved-clock schedule of the Mini-Fab's first five steps, for
`N` lots. Rounds are naturals; `start l k`/`finish l k` bracket lot `l`'s
step-`k` service; `ticks l k` are its tick rounds (strictly inside the
service window); `mach1 l`/`mach5 l` name the diffusion machine token held
during the step-1/step-5 service. Hypotheses = Lemma A/B/C premises +
diffusion capacity 2 (see module docstring). -/
structure MFSchedule (N : ℕ) where
  rel : Fin N → ℕ
  start : Fin N → Fin 5 → ℕ
  finish : Fin N → Fin 5 → ℕ
  ticks : Fin N → Fin 5 → Finset ℕ
  mach1 : Fin N → Bool
  mach5 : Fin N → Bool
  /-- the initial marking is in force at round 0; release fires ≥ 1. -/
  hrel : ∀ l, 1 ≤ rel l
  /-- token dwell: the q1 token from `rel` is consumable one round later. -/
  hstart1 : ∀ l, rel l + 1 ≤ start l 0
  /-- chain with dwell: finish k → start (k+1), strictly increasing rounds. -/
  hseq01 : ∀ l, finish l 0 + 1 ≤ start l 1
  hseq12 : ∀ l, finish l 1 + 1 ≤ start l 2
  hseq23 : ∀ l, finish l 2 + 1 ≤ start l 3
  hseq34 : ∀ l, finish l 3 + 1 ≤ start l 4
  /-- demand: each step fires exactly its `procTime` many ticks. -/
  hcard : ∀ l k, (ticks l k).card = procTime k
  /-- ticks sit strictly inside the service window. -/
  hwin : ∀ l k, ticks l k ⊆ Finset.Ioo (start l k) (finish l k)
  /-- METER LAW (one clock per schema): same-step ticks of distinct lots
  occupy distinct rounds. -/
  hmeter : ∀ (k : Fin 5) ⦃l l' : Fin N⦄, l ≠ l' → Disjoint (ticks l k) (ticks l' k)
  /-- diffusion capacity 2, assignment form: a step-1 service and a step-5
  service holding the same machine token are disjoint in time. -/
  hcap15 : ∀ l l', mach1 l = mach5 l' → finish l 0 < start l' 4 ∨ finish l' 4 < start l 0
  /-- two step-1 services on the same machine are disjoint. -/
  hcap11 : ∀ ⦃l l' : Fin N⦄, l ≠ l' → mach1 l = mach1 l' →
    finish l 0 < start l' 0 ∨ finish l' 0 < start l 0
  /-- two step-5 services on the same machine are disjoint. -/
  hcap55 : ∀ ⦃l l' : Fin N⦄, l ≠ l' → mach5 l = mach5 l' →
    finish l 4 < start l' 4 ∨ finish l' 4 < start l 4

namespace MFSchedule

variable {N : ℕ} (S : MFSchedule N)

lemma ticks_nonempty (l : Fin N) (k : Fin 5) : (S.ticks l k).Nonempty := by
  rw [← Finset.card_pos, S.hcard l k]
  exact procTime_pos k

/-- Window length: `procTime k` ticks strictly inside the window force
`finish ≥ start + procTime + 1`. -/
lemma finish_ge (l : Fin N) (k : Fin 5) :
    S.start l k + procTime k + 1 ≤ S.finish l k := by
  have h := Finset.card_le_card (S.hwin l k)
  rw [S.hcard l k, Nat.card_Ioo] at h
  have hpos := procTime_pos k
  omega

lemma start1_ge (l : Fin N) : 2 ≤ S.start l 0 := by
  have h1 := S.hrel l
  have h2 := S.hstart1 l
  omega

/-- First tick5 round of a lot. -/
def t5 (l : Fin N) : ℕ := (S.ticks l 4).min' (S.ticks_nonempty l 4)

lemma t5_mem (l : Fin N) : S.t5 l ∈ S.ticks l 4 := Finset.min'_mem _ _

lemma t5_min (l : Fin N) {r : ℕ} (hr : r ∈ S.ticks l 4) : S.t5 l ≤ r :=
  Finset.min'_le _ _ hr

lemma start5_lt_t5 (l : Fin N) : S.start l 4 < S.t5 l :=
  (Finset.mem_Ioo.mp (S.hwin l 4 (S.t5_mem l))).1

lemma t5_lt_finish5 (l : Fin N) : S.t5 l < S.finish l 4 :=
  (Finset.mem_Ioo.mp (S.hwin l 4 (S.t5_mem l))).2

/-- 255 distinct tick5 rounds from the first one: `finish5 ≥ t5 + 255`. -/
lemma finish5_ge (l : Fin N) : S.t5 l + 255 ≤ S.finish l 4 := by
  have hsub : S.ticks l 4 ⊆ Finset.Icc (S.t5 l) (S.finish l 4 - 1) := by
    intro r hr
    have h1 := S.t5_min l hr
    have h2 := (Finset.mem_Ioo.mp (S.hwin l 4 hr)).2
    rw [Finset.mem_Icc]
    omega
  have h := Finset.card_le_card hsub
  rw [S.hcard l 4, Nat.card_Icc, procTime_four] at h
  have h3 := S.t5_lt_finish5 l
  omega

/-- The +143 chain: `finish1 → start2 → … → start5 → first tick5` through
steps 2, 3, 4 with dwell. -/
lemma chain143 (l : Fin N) : S.finish l 0 + 143 ≤ S.t5 l := by
  have h01 := S.hseq01 l
  have h12 := S.hseq12 l
  have h23 := S.hseq23 l
  have h34 := S.hseq34 l
  have g1 := S.finish_ge l 1
  have g2 := S.finish_ge l 2
  have g3 := S.finish_ge l 3
  rw [procTime_one] at g1
  rw [procTime_two] at g2
  rw [procTime_three] at g3
  have g5 := S.start5_lt_t5 l
  omega

/-- Absolute prefix: `finish1 ≥ 228` (release ≥ 1, dwell, 225 ticks). -/
lemma finish1_ge (l : Fin N) : 228 ≤ S.finish l 0 := by
  have h := S.finish_ge l 0
  rw [procTime_zero] at h
  have h2 := S.start1_ge l
  omega

/-- Every lot's first tick5 is ≥ 371 (the 370-firing prefix). -/
lemma t5_ge (l : Fin N) : 371 ≤ S.t5 l := by
  have h1 := S.chain143 l
  have h2 := S.finish1_ge l
  omega

/-- Meter-window counting: the step-`k` tick sets of the lots in `A` are
pairwise disjoint (one clock per schema) of size `procTime k` each, so if
they all sit inside `[lo, hi]` then `|A| · procTime k ≤ hi + 1 - lo`. -/
lemma window_count (k : Fin 5) (A : Finset (Fin N)) (lo hi : ℕ)
    (hsub : ∀ l ∈ A, S.ticks l k ⊆ Finset.Icc lo hi) :
    A.card * procTime k ≤ hi + 1 - lo := by
  have hdisj : (A : Set (Fin N)).PairwiseDisjoint fun l => S.ticks l k := by
    intro l _ l' _ h
    exact S.hmeter k h
  have hU : (A.biUnion fun l => S.ticks l k) ⊆ Finset.Icc lo hi := by
    intro r hr
    obtain ⟨l, hl, hrl⟩ := Finset.mem_biUnion.mp hr
    exact hsub l hl hrl
  have h := Finset.card_le_card hU
  rw [Finset.card_biUnion hdisj, Finset.sum_const_nat fun l _ => S.hcard l k,
    Nat.card_Icc] at h
  exact h

/-- Entrant counting: if `l0` attains the minimum first tick5 among `A` and
all lots arrive by `B`, then the `255·|A|` distinct tick5 rounds of `A`'s
lots all sit in `[t5 l0, B-1]`, so `t5 l0 + 255·|A| ≤ B`. -/
lemma entrant_bound {B : ℕ} (hB : ∀ l, S.finish l 4 ≤ B)
    (A : Finset (Fin N)) (l0 : Fin N) (_hl0 : l0 ∈ A)
    (hmin : ∀ l ∈ A, S.t5 l0 ≤ S.t5 l) :
    S.t5 l0 + 255 * A.card ≤ B := by
  have hsub : ∀ l ∈ A, S.ticks l 4 ⊆ Finset.Icc (S.t5 l0) (B - 1) := by
    intro l hl r hr
    have h1 := S.t5_min l hr
    have h2 := (Finset.mem_Ioo.mp (S.hwin l 4 hr)).2
    have h3 := hB l
    have h4 := hmin l hl
    rw [Finset.mem_Icc]
    omega
  have h := S.window_count 4 A (S.t5 l0) (B - 1) hsub
  rw [procTime_four] at h
  have h5 : S.t5 l0 < B := lt_of_lt_of_le (S.t5_lt_finish5 l0) (hB l0)
  omega

/-- Step-1 window counting: all step-1 ticks of `A`'s lots are pairwise
distinct rounds in `[3, M-1]` whenever `M` bounds the step-1 finishes, so
`225·|A| ≤ M - 3`. -/
lemma window1_bound (A : Finset (Fin N)) (M : ℕ)
    (hM : ∀ l ∈ A, S.finish l 0 ≤ M) :
    225 * A.card ≤ M - 3 := by
  have hsub : ∀ l ∈ A, S.ticks l 0 ⊆ Finset.Icc 3 (M - 1) := by
    intro l hl r hr
    obtain ⟨h1, h2⟩ := Finset.mem_Ioo.mp (S.hwin l 0 hr)
    have h3 := S.start1_ge l
    have h4 := hM l hl
    rw [Finset.mem_Icc]
    omega
  have h := S.window_count 0 A 3 (M - 1) hsub
  rw [procTime_zero] at h
  omega

/-- **Forcing I** (collapses the tree's cases A/B2/B3): under all-arrivals
`≤ 255N + 374` with `N ≥ 3`, entrant 3's step-1 service overlaps both of
the two earliest step-5 services, so those share a diffusion machine and
the second entrant's first tick is `≥ 628` (a 257-round handover above
`t¹ ≥ 371`). Packages the entrants, their counting bounds, and `t³ ≥ 821`
for the `N ≥ 4` leg. -/
lemma forcing_one (hN : 3 ≤ N) {B : ℕ} (hB : ∀ l, S.finish l 4 ≤ B)
    (h374 : B ≤ 255 * N + 374) :
    ∃ l1 l2 l3 : Fin N, l1 ≠ l2 ∧ l1 ≠ l3 ∧ l2 ≠ l3 ∧
      S.t5 l1 + 255 * N ≤ B ∧
      S.t5 l2 + 255 * (N - 1) ≤ B ∧
      S.t5 l3 + 255 * (N - 2) ≤ B ∧
      628 ≤ S.t5 l2 ∧ 821 ≤ S.t5 l3 := by
  -- the three earliest entrants, by successive argmin of the first tick5
  have huniv : (Finset.univ : Finset (Fin N)).card = N := by simp
  have hu1 : (Finset.univ : Finset (Fin N)).Nonempty :=
    Finset.card_pos.mp (by omega)
  obtain ⟨l1, -, hmin1⟩ := Finset.exists_min_image Finset.univ S.t5 hu1
  have hA2card : (Finset.univ.erase l1).card = N - 1 := by
    have h := Finset.card_erase_of_mem (Finset.mem_univ l1)
    omega
  have hA2ne : (Finset.univ.erase l1).Nonempty := Finset.card_pos.mp (by omega)
  obtain ⟨l2, hl2mem, hmin2⟩ := Finset.exists_min_image _ S.t5 hA2ne
  have hne12 : l1 ≠ l2 := fun h => (Finset.mem_erase.mp hl2mem).1 h.symm
  have hA3card : ((Finset.univ.erase l1).erase l2).card = N - 2 := by
    have h := Finset.card_erase_of_mem hl2mem
    omega
  have hA3ne : ((Finset.univ.erase l1).erase l2).Nonempty :=
    Finset.card_pos.mp (by omega)
  obtain ⟨l3, hl3mem, hmin3⟩ := Finset.exists_min_image _ S.t5 hA3ne
  have hl3A2 : l3 ∈ Finset.univ.erase l1 := (Finset.mem_erase.mp hl3mem).2
  have hne23 : l2 ≠ l3 := fun h => (Finset.mem_erase.mp hl3mem).1 h.symm
  have hne13 : l1 ≠ l3 := fun h => (Finset.mem_erase.mp hl3A2).1 h.symm
  -- tick5-meter counting bounds (entrants k..N tick above t5 lk)
  have ht1 : S.t5 l1 + 255 * N ≤ B := by
    have h := S.entrant_bound hB Finset.univ l1 (Finset.mem_univ l1)
      fun l _ => hmin1 l (Finset.mem_univ l)
    rwa [huniv] at h
  have ht2 : S.t5 l2 + 255 * (N - 1) ≤ B := by
    have h := S.entrant_bound hB _ l2 hl2mem hmin2
    rwa [hA2card] at h
  have ht3 : S.t5 l3 + 255 * (N - 2) ≤ B := by
    have h := S.entrant_bound hB _ l3 hl3mem hmin3
    rwa [hA3card] at h
  have hT1 : S.t5 l1 ≤ 374 := by omega
  have hT2 : S.t5 l2 ≤ 629 := by omega
  have hT3 : S.t5 l3 ≤ 884 := by omega
  -- chain uppers on step-1 finishes
  have hc1 := S.chain143 l1
  have hc2 := S.chain143 l2
  have hc3 := S.chain143 l3
  -- tick1-meter window lowers: the step-1 ticks of the first k entrants are
  -- 225k distinct rounds starting at round 3
  have hQ2 : ({l1, l2} : Finset (Fin N)).card = 2 := Finset.card_pair hne12
  have hpair : 225 * ({l1, l2} : Finset (Fin N)).card ≤
      max (S.finish l1 0) (S.finish l2 0) - 3 := by
    refine S.window1_bound _ _ ?_
    intro l hl
    rcases Finset.mem_insert.mp hl with rfl | hl
    · exact le_max_left _ _
    · rw [Finset.mem_singleton] at hl
      subst hl
      exact le_max_right _ _
  rw [hQ2] at hpair
  have hf12 : 453 ≤ S.finish l2 0 := by
    rcases le_total (S.finish l1 0) (S.finish l2 0) with hle | hle
    · rw [max_eq_right hle] at hpair
      omega
    · rw [max_eq_left hle] at hpair
      omega
  have hQ3 : ({l1, l2, l3} : Finset (Fin N)).card = 3 := by
    rw [Finset.card_insert_of_notMem (by simp [hne12, hne13]),
      Finset.card_pair hne23]
  have htriple : 225 * ({l1, l2, l3} : Finset (Fin N)).card ≤
      max (S.finish l1 0) (max (S.finish l2 0) (S.finish l3 0)) - 3 := by
    refine S.window1_bound _ _ ?_
    intro l hl
    simp only [Finset.mem_insert, Finset.mem_singleton] at hl
    rcases hl with rfl | rfl | rfl
    · exact le_max_left _ _
    · exact le_max_of_le_right (le_max_left _ _)
    · exact le_max_of_le_right (le_max_right _ _)
  rw [hQ3] at htriple
  have hf13 : 678 ≤ S.finish l3 0 := by
    rcases le_total (S.finish l1 0) (max (S.finish l2 0) (S.finish l3 0)) with h | h
    · rw [max_eq_right h] at htriple
      rcases le_total (S.finish l2 0) (S.finish l3 0) with h2 | h2
      · rw [max_eq_right h2] at htriple
        omega
      · rw [max_eq_left h2] at htriple
        omega
    · rw [max_eq_left h] at htriple
      omega
  have h821 : 821 ≤ S.t5 l3 := by omega
  -- service geometry at the diffusion station
  have hs51 := S.start5_lt_t5 l1
  have hs52 := S.start5_lt_t5 l2
  have hF51 := S.finish5_ge l1
  have hF52 := S.finish5_ge l2
  have hlen3 := S.finish_ge l3 0
  rw [procTime_zero] at hlen3
  have ht5lo1 := S.t5_ge l1
  -- entrant 3's step-1 service overlaps S5(l1): [s1₃, f1₃] ∋ [·,678⁺] meets
  -- [s5₁, f5₁] ⊇ [≤373, ≥626]
  have hm31 : ¬S.mach1 l3 = S.mach5 l1 := by
    intro heq
    rcases S.hcap15 l3 l1 heq with h | h <;> omega
  -- … and overlaps S5(l2): [s5₂, f5₂] ⊇ [≤628, ≥851]
  have hm32 : ¬S.mach1 l3 = S.mach5 l2 := by
    intro heq
    rcases S.hcap15 l3 l2 heq with h | h <;> omega
  -- two machines: the early step-5 services share one
  have hm512 : S.mach5 l1 = S.mach5 l2 := by
    cases hb : S.mach1 l3 <;> cases hb1 : S.mach5 l1 <;> cases hb2 : S.mach5 l2 <;>
      simp_all
  -- same machine ⟹ disjoint services ⟹ 257-round handover
  have hT21 : S.t5 l1 + 257 ≤ S.t5 l2 := by
    rcases S.hcap55 hne12 hm512 with h | h <;> omega
  have h628 : 628 ≤ S.t5 l2 := by omega
  exact ⟨l1, l2, l3, hne12, hne13, hne23, ht1, ht2, ht3, h628, h821⟩

/-- **The N ≥ 3 floor, strengthened by one forcing step**: every schedule of
`N ≥ 3` lots meeting the meter law, the precedence chains, and diffusion
capacity 2 has all-arrivals makespan `≥ 255N + 373`. Tight at `N = 3`:
the solver's two-sided optimum is `T_opt(3) = 1138 = 255·3 + 373`. -/
theorem floor_contiguity_three (hN : 3 ≤ N) {B : ℕ}
    (hB : ∀ l, S.finish l 4 ≤ B) : 255 * N + 373 ≤ B := by
  by_cases h374 : B ≤ 255 * N + 374
  · obtain ⟨l1, l2, l3, -, -, -, -, ht2, -, h628, -⟩ := S.forcing_one hN hB h374
    omega
  · omega

/-- **Theorem D, mechanized (the machine-contiguity floor)**: every schedule
of `N ≥ 4` lots meeting the meter law, the precedence chains, and diffusion
capacity 2 has all-arrivals makespan `≥ 255N + 375`. With the validated
attaining family π* this pins the exact optimum `T_opt(N) = 255N + 375` for
all `N ≥ 4` (solver-certified two-sidedly at `N = 4..7`). -/
theorem floor_contiguity (hN : 4 ≤ N) {B : ℕ}
    (hB : ∀ l, S.finish l 4 ≤ B) : 255 * N + 375 ≤ B := by
  by_contra hlt
  have h374 : B ≤ 255 * N + 374 := by omega
  obtain ⟨l1, l2, l3, hne12, hne13, hne23, ht1, ht2, ht3, h628, h821⟩ :=
    S.forcing_one (by omega) hB h374
  -- the fourth entrant
  have huniv : (Finset.univ : Finset (Fin N)).card = N := by simp
  have hl2mem : l2 ∈ Finset.univ.erase l1 :=
    Finset.mem_erase.mpr ⟨hne12.symm, Finset.mem_univ l2⟩
  have hl3mem : l3 ∈ (Finset.univ.erase l1).erase l2 :=
    Finset.mem_erase.mpr ⟨hne23.symm, Finset.mem_erase.mpr ⟨hne13.symm, Finset.mem_univ l3⟩⟩
  have hA4card : (((Finset.univ.erase l1).erase l2).erase l3).card = N - 3 := by
    have h3 := Finset.card_erase_of_mem hl3mem
    have h2 := Finset.card_erase_of_mem hl2mem
    have h1 := Finset.card_erase_of_mem (Finset.mem_univ l1)
    omega
  have hA4ne : (((Finset.univ.erase l1).erase l2).erase l3).Nonempty :=
    Finset.card_pos.mp (by omega)
  obtain ⟨l4, hl4mem, hmin4⟩ := Finset.exists_min_image _ S.t5 hA4ne
  have hne34 : l4 ≠ l3 := (Finset.mem_erase.mp hl4mem).1
  have hl4A3 : l4 ∈ (Finset.univ.erase l1).erase l2 := (Finset.mem_erase.mp hl4mem).2
  have hne24 : l4 ≠ l2 := (Finset.mem_erase.mp hl4A3).1
  have hne14 : l4 ≠ l1 := (Finset.mem_erase.mp (Finset.mem_erase.mp hl4A3).2).1
  have ht4 : S.t5 l4 + 255 * (N - 3) ≤ B := by
    have h := S.entrant_bound hB _ l4 hl4mem hmin4
    rwa [hA4card] at h
  -- numeric bounds
  have hT2 : S.t5 l2 ≤ 629 := by omega
  have hT3 : S.t5 l3 ≤ 884 := by omega
  have hT4 : S.t5 l4 ≤ 1139 := by omega
  have hc1 := S.chain143 l1
  have hc2 := S.chain143 l2
  have hc3 := S.chain143 l3
  have hc4 := S.chain143 l4
  have hT1 : S.t5 l1 ≤ 374 := by omega
  -- tick1-meter window over the four entrants: f1₄ ≥ 903
  have hQ4 : ({l1, l2, l3, l4} : Finset (Fin N)).card = 4 := by
    rw [Finset.card_insert_of_notMem (by simp [hne12, hne13, hne14.symm]),
      Finset.card_insert_of_notMem (by simp [hne23, hne24.symm]),
      Finset.card_pair hne34.symm]
  have hquad : 225 * ({l1, l2, l3, l4} : Finset (Fin N)).card ≤
      max (S.finish l1 0)
        (max (S.finish l2 0) (max (S.finish l3 0) (S.finish l4 0))) - 3 := by
    refine S.window1_bound _ _ ?_
    intro l hl
    simp only [Finset.mem_insert, Finset.mem_singleton] at hl
    rcases hl with rfl | rfl | rfl | rfl
    · exact le_max_left _ _
    · exact le_max_of_le_right (le_max_left _ _)
    · exact le_max_of_le_right (le_max_of_le_right (le_max_left _ _))
    · exact le_max_of_le_right (le_max_of_le_right (le_max_right _ _))
  rw [hQ4] at hquad
  have hf14 : 903 ≤ S.finish l4 0 := by
    rcases le_total (S.finish l1 0)
      (max (S.finish l2 0) (max (S.finish l3 0) (S.finish l4 0))) with h | h
    · rw [max_eq_right h] at hquad
      rcases le_total (S.finish l2 0) (max (S.finish l3 0) (S.finish l4 0)) with h2 | h2
      · rw [max_eq_right h2] at hquad
        rcases le_total (S.finish l3 0) (S.finish l4 0) with h3 | h3
        · rw [max_eq_right h3] at hquad
          omega
        · rw [max_eq_left h3] at hquad
          omega
      · rw [max_eq_left h2] at hquad
        omega
    · rw [max_eq_left h] at hquad
      omega
  -- service geometry
  have hs52 := S.start5_lt_t5 l2
  have hs53 := S.start5_lt_t5 l3
  have hF52 := S.finish5_ge l2
  have hF53 := S.finish5_ge l3
  have hlen4 := S.finish_ge l4 0
  rw [procTime_zero] at hlen4
  -- entrant 4's step-1 service overlaps S5(l2): [s5₂, f5₂] ⊇ [≤628, ≥883]
  have hm42 : ¬S.mach1 l4 = S.mach5 l2 := by
    intro heq
    rcases S.hcap15 l4 l2 heq with h | h <;> omega
  -- … and overlaps S5(l3): [s5₃, f5₃] ⊇ [≤883, ≥1076]
  have hm43 : ¬S.mach1 l4 = S.mach5 l3 := by
    intro heq
    rcases S.hcap15 l4 l3 heq with h | h <;> omega
  -- two machines: the second and third step-5 services share one
  have hm523 : S.mach5 l2 = S.mach5 l3 := by
    cases hb : S.mach1 l4 <;> cases hb2 : S.mach5 l2 <;> cases hb3 : S.mach5 l3 <;>
      simp_all
  -- second 257-round handover: t³ ≥ 628 + 257 = 885, against the meter
  -- budget t³ ≤ 884 — the contradiction that buys the +4
  have hT32 : S.t5 l2 + 257 ≤ S.t5 l3 := by
    rcases S.hcap55 hne23 hm523 with h | h <;> omega
  omega

/-- Anchor N = 4: `T_opt(4) = 1395` (solver: UNSAT at 1394, validated SAT
at 1395) — the certified floor matches exactly. -/
theorem anchor_four (W : MFSchedule 4) {B : ℕ} (hB : ∀ l, W.finish l 4 ≤ B) :
    1395 ≤ B := by
  have h := W.floor_contiguity (by norm_num) hB
  omega

/-- Anchor N = 5: `T_opt(5) = 1650`. -/
theorem anchor_five (W : MFSchedule 5) {B : ℕ} (hB : ∀ l, W.finish l 4 ≤ B) :
    1650 ≤ B := by
  have h := W.floor_contiguity (by norm_num) hB
  omega

/-- Anchor N = 6: `T_opt(6) = 1905`. -/
theorem anchor_six (W : MFSchedule 6) {B : ℕ} (hB : ∀ l, W.finish l 4 ≤ B) :
    1905 ≤ B := by
  have h := W.floor_contiguity (by norm_num) hB
  omega

/-- Anchor N = 7: `T_opt(7) = 2160`. -/
theorem anchor_seven (W : MFSchedule 7) {B : ℕ} (hB : ∀ l, W.finish l 4 ≤ B) :
    2160 ≤ B := by
  have h := W.floor_contiguity (by norm_num) hB
  omega

/-- Anchor N = 3: `T_opt(3) = 1138 = 255·3 + 373` — the N ≥ 3 floor is
tight there too. -/
theorem anchor_three (W : MFSchedule 3) {B : ℕ} (hB : ∀ l, W.finish l 4 ≤ B) :
    1138 ≤ B := by
  have h := W.floor_contiguity_three (by norm_num) hB
  omega

end MFSchedule

end BernsteinLogic.MiniFab
