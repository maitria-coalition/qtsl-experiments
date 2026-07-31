# The Intel Five-Machine Six-Step Mini-Fab — spec digest (Kempf 1994)

*Spec digest, 2026-07-19. Primary witness read for this digest: van den Berk, "Analysis of the Intel
Five-Machine Six Step Mini-Fab" (TU/e Systems Engineering masters
thesis, May 2004, supervised Rooda/Lefeber;
`dc.wtb.tue.nl/lefeber/do_download_pdf.php?id=74`, 103 pp, fetched
2026-07-19) — chapter 2 is a complete restatement of the Kempf case and
every number below cites it. A prior survey confirmed the model's
identity; one parenthetical there is CORRECTED below.*

## Identity and the census correction

Three workstations, five machines, six route steps, re-entrant flow
(each station visited twice). Machine counts per the primary source
(TU/e §2.1):

| station | machines | buffer capacity (lots) | batching |
|---|---|---|---|
| diffusion | **2** (ASU convention: Ma, Mb) | 18 | batches of 3 |
| implantation | **2** (Mc, Md) | 12 | none |
| lithography | **1** (Me) | 12 | none |

The scoping census's parenthetical read "diffusion (2 machines,
batching), implant (1, the bottleneck), lithography (2)" — the machine
counts for implant and litho are swapped there. The primary source has
implantation with two machines and **lithography as the single-machine
station**, and the capacity arithmetic below confirms litho as the
bottleneck. The census's 3/5/6 structure and everything else stands.

## Products and release

Three product types; weekly release (week = $7 \times 1440 = 10080$
min), released at constant per-type intervals (TU/e §2.1, §3.2 —
signal-generator idiom):

| type | lots/week | release interval (min) |
|---|---|---|
| Test | 3 | $3360$ |
| A | 51 | $10080/51 = 3360/17$ |
| B | 30 | $336$ |

Total 84 lots/week. Note the type-A interval is a non-integer rational
— exact-$\mathbb{Q}$ time arithmetic is the honest representation (the
thesis sets the χ time unit to 1 min and calls all *parameters*
integers; the A-interval shows event times still need $\mathbb{Q}$).
Arrivals are **deterministic** in the spec — the only stochastic
elements are implant failures/repairs (below).

## Route and processing times

Six steps, sequence diffusion → implantation → lithography →
implantation → diffusion → lithography (TU/e §2.1). Each machine runs
a "low" recipe for a lot's first visit to its station and a "high"
recipe for the second:

| step | station | recipe | process time (min) |
|---|---|---|---|
| 1 | diffusion | low | 225 |
| 2 | implantation | low | 30 |
| 3 | lithography | low | 55 |
| 4 | implantation | high | 50 |
| 5 | diffusion | high | 255 |
| 6 | lithography | high | 10 |

No preemption, no rework.

## Machine-input restrictions (TU/e §2.2)

- **Test-product machine dedication**: at the two-machine stations, a
  test lot's second visit must use the machine it did NOT use on its
  first visit (so both machines are monitored).
- **Diffusion batching**: machine input is a batch of exactly 3 lots,
  all of the same step (all step-1 or all step-5); a batch contains at
  most one test lot ("less than two"); a **high**-step batch must not
  mix commercial types (no {A,B} together; test may join either).

## Transport (TU/e §2.2)

One transporter, capacity 1 lot; 4 min per move between adjacent
areas (empty or loaded), 1 min load, 1 min unload. Area line order
per the case figures: start, diffusion, lithography, implantation,
finish.

## Operators (TU/e §2.3)

Two operators. Operator 0 serves diffusion + implantation; operator 1
serves lithography + implantation. Tasks (non-preemptive; 1 min move
between adjacent stations):

| station | load (min) | unload (min) |
|---|---|---|
| diffusion | 20 | 40 |
| lithography | 10 | 10 |
| implantation | 15 | 15 |

Lithography setup, performed by an operator immediately before
loading, machine empty during setup, required when machine input
changes vs the previous job: product-type change 5 min, step change
10 min, both 12 min. Off time per 720-min shift, per operator: two
60-min breaks + one 60-min meeting, schedule otherwise unconstrained.

## Technician and maintenance (TU/e §2.4)

One technician (non-preemptive; two 45-min breaks + one 30-min
meeting per shift; 1 min moves).

- **Scheduled maintenance** (machine empty throughout): diffusion 75
  min per machine with window opening at start of day or 720 min
  after the previous maintenance (whichever later), closing at end of
  day; lithography 30 min and implantation 120 min with windows per
  *shift* (open at shift start or 360 min after previous, close at
  shift end). A machine whose window closed unserved finishes its
  current cycle then refuses input until maintained.
- **Unscheduled maintenance**: implantation machines fail during
  processing; the in-process lot is **wasted**; repair time uniform
  $420 \pm 60$ min, time between failures uniform $3000 \pm 1560$ min
  (the thesis states "stochastically distributed ... $\pm$"; the
  literature's usual reading is uniform on the stated interval).
  Scheduled and unscheduled maintenance are independent.

## Reference numbers for any differential check (TU/e chapters 3–5)

All for the FULL model (transport + operators + technician +
failures), χ discrete-event simulation:

- With FIFO control everywhere and the required release of 84
  lots/week, the line is **unstable** (WIP grows without bound);
  productivity 81.5%.
- **FIFO maximal throughput $\delta_{\max} = 57.5$ lots/week**
  (release scaled at fixed mix, buffer capacity constraints
  neglected), with **mean flow time $\varphi = 4580$ min** at that
  release rate.
- Capacity analysis (their Appendix H): all resources sufficient for
  84/week; a manually constructed cyclic schedule of one shift (6
  lots/shift $\times$ 14 = 84/week) fits in 700 of 720 min, so the
  deterministic control problem is feasible; the litho machine has at
  least three 5-min idles per shift forced by operator-1 off time.
- With their improved (pull-class) control: required throughput met
  when break-downs are neglected; with break-downs, max throughput
  80.4 lots/week (litho losing throughput to blocked personnel), i.e.
  84/week is met only "perhaps" with further-designed control.

Bottleneck arithmetic (deterministic, machine-occupancy only, per
lot): litho $= (10 + 55 + 10) + (10 + 10 + 10) = 105$ min on 1
machine; diffusion $= (20 + 225 + 40)/3 + (20 + 255 + 40)/3 = 95 +
105 = 200$ min per lot spread over 2 machines $= 100$; implantation
$= (15+30+15) + (15+50+15) = 140$ over 2 machines $= 70$. At 84
lots/week litho runs $84 \times 105 / 10080 = 87.5\%$ before setups
(~24 min/shift, their Appendix H average) and maintenance, diffusion
$83.3\%$, implantation $58.3\%$ — the single-machine litho station is
the bottleneck. *(Emendation 2026-07-20, during the fluid-certificate pass: this
paragraph originally summed the diffusion visits to "105 min ... = 52.5"
— an arithmetic slip; $95 + 105 = 200$ over 2 machines $= 100$,
matching the sim README's measured $83.3\%$ diffusion occupancy.
Bottleneck identity unaffected.)*

## Underdetermined by the public spec (decision points a rendering must carry)

1. **Diffusion load/unload granularity**: 20/40 min per *batch* or
   per *lot*. The batch is the machine-input unit, so per-batch is
   the natural reading (taken here); the thesis text says "loading
   times of the ... process," which does not disambiguate by itself.
2. **Failure clock basis**: time-between-failures counts processing
   time only ("machines break down during processing") or wall time.
   Taken here: the failure clock runs during processing only.
3. **Break/meeting placement** within a shift is explicitly free
   ("off time is unrestricted to synchronization") — a policy choice
   that visibly moves $\delta_{\max}$ (the thesis's improvement
   chapter re-schedules it).
4. **Distribution family** for repair/TBF: uniform assumed (see
   above).
5. **Dispatch beyond FIFO**: which enabled colour class a freed
   machine/operator serves next; FIFO fixes within-class order but
   the thesis's own control chapter shows the class-level choice is
   the whole game.
6. Thesis wart: §4.1 says "operator 2" once; only operators 0 and 1
   exist (context implies operator 1, the litho-side operator).
