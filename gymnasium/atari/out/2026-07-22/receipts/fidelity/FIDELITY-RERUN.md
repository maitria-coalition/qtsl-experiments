# Fidelity-tier re-run (posture 2: measured model gap vs ALE), 2026-07-22

Protocol: identical to the published receipts (README "Differential receipts") —
ale-py 0.12.0, sticky actions off, frameskip 1, seed 42, boot `ale`
(2000-clk reset hold, 60 NOOP frames, two 8-frame RESET presses), RAM at
VSYNC rising edge, circuit RAM zero-init, **inputs exactly as declared in the
claim data: switches=15, joy1=127, paddle=[132,132,0,0]**. Machine: gb10
(GB10), sim rebuilt from the workshop tree with Verilator 5.020 (published
builds used the oss-cad-suite verilator; no behavioral delta observed —
all divergence signatures reproduce exactly). ROMs bit-identical to ale-py's
bundled images (md5s in PIN-MANIFEST.txt).

| published number | re-derived 2026-07-22 | verdict |
|---|---|---|
| Pong 1000f NOOP: return -6 / -6 | ALE -6 / CIRC -6 | match |
| Pong: 6 reward events each side, every event +2 frames in circuit (255 vs 257 … 955 vs 957) | ALE {255,395,535,675,815,955}, CIRC {257,397,537,677,817,957} | match, exactly |
| Pong: rally period exactly 140 frames on both sides | inter-event gaps [140,140,140,140,140] both sides | match, exactly |
| Pong persistent divergent addrs {8,21,49,50,54} | {8,21,49,50,54} (5/128, 100%-of-frames) | match, exactly |
| Freeway 1000f all-UP: return +3 / +3 | ALE 3 / CIRC 3 | match |
| Freeway: first crossing SAME frame (171); later +1 (437 vs 438, 637 vs 638) | ALE {171,437,637}, CIRC {171,438,638} | match, exactly |
| Freeway persistent divergent addrs {1,2,14,18,19,121,122} | {1,2,14,18,19,121,122} (7/128) | match, exactly |
| Freeway 600f pseudorandom walk: 0 / 0, same divergence family | 0 / 0, reward streams IDENTICAL; persistent family {1,2,14,18,19,121,122} + car-lane {119,120} | match |
| "116–123 of 128 RAM bytes exact per frame" | measured full band: pong [115,127] (single 115-frame; >99.8% of frames ≥118), freeway [114,124] (three frames <116) | match as a typical-range summary; strict per-frame min is 114–115 on 1–3 frames per run — the published band reads as the central band, not strict min/max |

Raw per-run atrcdiff JSONs: `diff-pong.json`, `diff-freeway.json`,
`diff-freeway-walk.json` (this directory).

Reproduction finding (banked for the chapter/README): the operative input
calibration is the four-tuple `paddle=[132,132,0,0]` declared in the claim
data. The README prose says "pot=132"; the sim's built-in default is 128,
and runs at [128,128,128,128] or [132,132,132,132] produce 14 and 10
persistent divergent addresses respectively (the paddle family returns).
Only the claim's exact tuple reproduces the published 5-address set — one
more instance of the claim being the protocol record, and worth one README
sentence at the next editorial pass.
