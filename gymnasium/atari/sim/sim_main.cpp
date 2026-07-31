// Verilator driver for the AtariCircuit env (sim_top.v).
//
// Modes:
//   batch:       --script actions.txt --trace out.atrc [--frames N]
//   interactive: --interactive   (line protocol on stdin/stdout, for the gym adapter)
//
// Boot protocol (--boot ale, default) mirrors ALE's StellaEnvironment::reset():
//   hard reset (reset held HOLD_CLKS, RAM zero-initialized via debug port)
//   -> 60 frames NOOP -> 4 frames console-RESET -> 4 frames console-RESET
//   (two softResets; ALE runs settings->reset()+setMode between them, which
//   emulates nothing for Pong/Freeway; startingActions are empty for both).
//
// A "frame" is delimited by the TIA VSYNC rising edge, matching ALE's frame
// accounting (Stella ends a frame when the game asserts VSYNC). RAM is
// sampled at the posedge that observes the rise.
//
// Trace format (.atrc v1, little-endian):
//   header: "ATRC" u8 ver=1 u8 rec=132 u16 pad=0 u64 pad=0   (16 bytes)
//   record: u8 ram[128], i32 reward                          (132 bytes/frame)
//
// Reward decode (--reward): pong: d(ram[14]-ram[13]); freeway: clamp01 of
// d(BCD ram[103]); montezuma: d of BCD(ram[19])*10000 + BCD(ram[20])*100 +
// BCD(ram[21]) (ALE reads 0x95/0x94/0x93 = bytes 21/20/19 as lower/middle/
// higher — v0.12.0 MontezumaRevenge.cpp:45 + RomUtils.cpp getDecimalScore);
// none: 0. Decodes mirror ALE's games/supported/*.cpp.
//
// E0 hotspot coverage — every clk with the cart-access strobe
// asserted and rom_address in $FE0-$FF7 is counted per address; a nonzero
// histogram prints to stderr at exit (the differential witness set for the
// banked-cart mapper).
//
// --framedump <path>: writes the assembled pixel stream (.afrm v1):
//   header: "AFRM" u8 ver=1 u8 src(1=circuit-sim) u16 W=160 u16 H=239
//           u16 y0=22 u32 pad=0                                  (16 bytes)
//   record: u8 pixels[H*W], row-major, row 0 = TIA ypos 22       (per frame)
// Pixel values are the TIA's 7-bit {color,luma} (COLUxx >> 1). Decode rule,
// derived from TIA.v's clocked block (video is REGISTERED: vid_out after a
// posedge holds the pixel at the (xpos,ypos) the counters showed BEFORE that
// posedge — pair video@t+1 with coords@t):
//   a pixel was drawn on a tick  <=>  (xpos,ypos) changed across the tick
//                                     AND post-coords != (0,0)
//                                     AND pre-ypos in [22,260] AND pre-xpos < 160
// (coords advance exactly on TIA-enable ticks with ypos<261; the only other
// coord change is the cpu-tick VSYNC-write reset, which always lands on
// (0,0), and the one enable-path arrival at (0,0) — the ypos-261 wrap with
// xpos=0 — has invisible pre-coords, so excluding post==(0,0) is exact.)
// One frame record is emitted per Run-phase frame at the same cadence as the
// .atrc trace records; the buffer persists across frames (Stella-like).
// Incompatible with --sample-delay > 0 (post-rise ticks would bleed the next
// frame's pixels into the snapshot): refused at startup.

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cstdint>
#include <string>
#include <vector>
#include <verilated.h>
#include "Vsim_top.h"

static const int HOLD_CLKS = 2000;        // reset hold (matches upstream harness)
static const long FIRST_VSYNC_TIMEOUT = 20000000; // clks allowed before first vsync
static const long FRAME_TIMEOUT = 1600000;        // ~20x nominal frame (79648 clks)

// pixel-assembly constants (see decode-rule comment above).
static const int FD_W = 160;   // visible pixels per line
static const int FD_H = 239;   // TIA ypos 22..260 inclusive
static const int FD_Y0 = 22;   // NTSC `start` in TIA.v
static const int FD_YMAX = 260;

struct Sim {
    Vsim_top* top;
    long clk_count = 0;
    long frame_count = 0;
    int prev_vsync = 0;
    long sample_delay = 0; // extra clks past the vsync rise before RAM sampling
    uint64_t hotspot_hits[24] = {0}; // cart accesses at $FE0..$FF7 (E0 witness set)
    bool fd_enabled = false;
    uint8_t fbuf[FD_H * FD_W] = {0}; // persists across frames (Stella-like)

    explicit Sim(Vsim_top* t) : top(t) {}

    void tick() {
        top->clk = 0; top->eval();
        unsigned px = top->xpos, py = top->ypos; // coords BEFORE the posedge
        top->clk = 1; top->eval();
        clk_count++;
        if (fd_enabled) {
            unsigned nx = top->xpos, ny = top->ypos;
            if ((nx != px || ny != py) && !(nx == 0 && ny == 0)
                && py >= (unsigned)FD_Y0 && py <= (unsigned)FD_YMAX
                && px < (unsigned)FD_W)
                fbuf[(py - FD_Y0) * FD_W + px] = top->video; // registered: pixel@pre-coords
        }
        if (top->dbg_rom_read) {
            unsigned a = top->dbg_rom_address;
            if (a >= 0xFE0 && a <= 0xFF7) hotspot_hits[a - 0xFE0]++;
        }
    }

    void report_hotspots() {
        uint64_t total = 0;
        for (int i = 0; i < 24; i++) total += hotspot_hits[i];
        if (!total) return;
        fprintf(stderr, "HOTSPOTS (cart accesses in $1FE0-$1FF7):");
        for (int i = 0; i < 24; i++)
            if (hotspot_hits[i])
                fprintf(stderr, " 1f%x:%llu", 0xe0 + i,
                        (unsigned long long)hotspot_hits[i]);
        fprintf(stderr, "\n");
    }

    // Advance until the next vsync rising edge (+ sample_delay clks).
    // Returns clks consumed, or -1 on timeout.
    long run_frame(long timeout) {
        long start = clk_count;
        while (clk_count - start < timeout) {
            tick();
            int vs = top->vsync;
            if (vs && !prev_vsync) {
                prev_vsync = vs;
                frame_count++;
                for (long d = 0; d < sample_delay; d++) tick();
                prev_vsync = top->vsync; // resync after delayed run
                return clk_count - start;
            }
            prev_vsync = vs;
        }
        return -1;
    }

    void read_ram(uint8_t out[128]) {
        for (int i = 0; i < 128; i++)
            out[i] = (top->ram_flat[i / 4] >> ((i % 4) * 8)) & 0xff;
    }

    void hard_reset_and_zero_ram() {
        top->reset = 1;
        top->joy0 = 0x7f; top->joy1 = 0x7f;
        top->switches = 0xf;   // color=1, both difficulty B, sw3=1
        top->paddle0 = 128;
        top->ram_we = 0;
        for (int i = 0; i < HOLD_CLKS; i++) {
            if (i < 128) { top->ram_we = 1; top->ram_waddr = i; top->ram_wdata = 0; }
            else top->ram_we = 0;
            tick();
        }
        top->ram_we = 0;
        top->reset = 0;
        prev_vsync = 0;
        frame_count = 0;
    }
};

// ALE Action enum -> joystick bits. joy0: {RIGHT,LEFT,DOWN,UP,SELECT,FIRE,RESET}, active-low.
static uint8_t action_to_joy(int a, bool console_reset) {
    bool up=false, down=false, left=false, right=false, fire=false;
    switch (a) {
        case 0: break;                                   // NOOP
        case 1: fire=true; break;                        // FIRE
        case 2: up=true; break;
        case 3: right=true; break;
        case 4: left=true; break;
        case 5: down=true; break;
        case 6: up=true; right=true; break;
        case 7: up=true; left=true; break;
        case 8: down=true; right=true; break;
        case 9: down=true; left=true; break;
        case 10: up=true; fire=true; break;
        case 11: right=true; fire=true; break;
        case 12: left=true; fire=true; break;
        case 13: down=true; fire=true; break;
        case 14: up=true; right=true; fire=true; break;
        case 15: up=true; left=true; fire=true; break;
        case 16: down=true; right=true; fire=true; break;
        case 17: down=true; left=true; fire=true; break;
        default: break; // player-B / RESET(40) handled by console_reset path
    }
    uint8_t joy = 0x7f;
    if (right) joy &= ~(1 << 6);
    if (left)  joy &= ~(1 << 5);
    if (down)  joy &= ~(1 << 4);
    if (up)    joy &= ~(1 << 3);
    if (fire)  joy &= ~(1 << 1);
    if (console_reset) joy &= ~(1 << 0);
    return joy;
}

enum RewardMode { R_NONE, R_PONG, R_FREEWAY, R_MONTEZUMA };

static inline int bcd2(uint8_t b) { return (b >> 4) * 10 + (b & 0xf); }

struct RewardState {
    RewardMode mode = R_NONE;
    int prev_score = 0;
    bool primed = false;
    int step(const uint8_t ram[128]) {
        int score = 0;
        switch (mode) {
            case R_PONG:    score = (int)ram[14] - (int)ram[13]; break;
            case R_FREEWAY: score = bcd2(ram[103]); break;
            case R_MONTEZUMA:
                score = bcd2(ram[19]) * 10000 + bcd2(ram[20]) * 100 + bcd2(ram[21]);
                break;
            case R_NONE:    return 0;
        }
        int r = primed ? score - prev_score : 0;
        prev_score = score; primed = true;
        if (mode == R_FREEWAY) { if (r < 0) r = 0; if (r > 1) r = 1; }
        return r;
    }
};

static void write_trace_header(FILE* f) {
    uint8_t h[16] = {0};
    memcpy(h, "ATRC", 4); h[4] = 1; h[5] = 132;
    fwrite(h, 1, 16, f);
}

static void write_afrm_header(FILE* f) {
    uint8_t h[16] = {0};
    memcpy(h, "AFRM", 4);
    h[4] = 1;                       // ver
    h[5] = 1;                       // src: 1 = circuit-sim
    h[6] = FD_W & 0xff;  h[7] = FD_W >> 8;   // u16 LE width
    h[8] = FD_H & 0xff;  h[9] = FD_H >> 8;   // u16 LE height
    h[10] = FD_Y0 & 0xff; h[11] = FD_Y0 >> 8; // u16 LE y0 (TIA ypos of row 0)
    fwrite(h, 1, 16, f);
}

static void hexdump_ram(const uint8_t ram[128], char out[257]) {
    static const char* d = "0123456789abcdef";
    for (int i = 0; i < 128; i++) { out[2*i] = d[ram[i] >> 4]; out[2*i+1] = d[ram[i] & 0xf]; }
    out[256] = 0;
}

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);

    std::string script_path, trace_path, framedump_path, reward_name = "none", boot = "ale";
    bool interactive = false;
    long max_frames = -1;
    int paddle[4] = {128, 128, 128, 128};
    long sample_delay = 0;
    for (int i = 1; i < argc; i++) {
        std::string a = argv[i];
        auto next = [&]() -> std::string { return (i + 1 < argc) ? argv[++i] : ""; };
        if (a == "--script") script_path = next();
        else if (a == "--trace") trace_path = next();
        else if (a == "--framedump") framedump_path = next();
        else if (a == "--reward") reward_name = next();
        else if (a == "--boot") boot = next();
        else if (a == "--frames") max_frames = atol(next().c_str());
        else if (a == "--paddle0") paddle[0] = atoi(next().c_str());
        else if (a == "--paddle1") paddle[1] = atoi(next().c_str());
        else if (a == "--paddle2") paddle[2] = atoi(next().c_str());
        else if (a == "--paddle3") paddle[3] = atoi(next().c_str());
        else if (a == "--sample-delay") sample_delay = atol(next().c_str());
        else if (a == "--interactive") interactive = true;
    }
    if (!framedump_path.empty() && sample_delay > 0) {
        fprintf(stderr, "FATAL: --framedump is incompatible with --sample-delay > 0 "
                        "(post-rise ticks bleed the next frame's pixels)\n");
        return 2;
    }
    if (!framedump_path.empty() && interactive) {
        fprintf(stderr, "FATAL: --framedump is batch-mode only\n");
        return 2;
    }

    auto reward_mode = [](const std::string& name) -> RewardMode {
        if (name == "pong") return R_PONG;
        if (name == "freeway") return R_FREEWAY;
        if (name == "montezuma") return R_MONTEZUMA;
        return R_NONE;
    };
    RewardState rew;
    rew.mode = reward_mode(reward_name);

    Vsim_top* top = new Vsim_top;
    Sim sim(top);
    sim.sample_delay = sample_delay;
    sim.fd_enabled = !framedump_path.empty();
    top->paddle0 = paddle[0]; top->paddle1 = paddle[1];
    top->paddle2 = paddle[2]; top->paddle3 = paddle[3];

    auto boot_ale = [&]() -> bool {
        sim.hard_reset_and_zero_ram();
        top->paddle0 = paddle[0]; top->paddle1 = paddle[1];
        top->paddle2 = paddle[2]; top->paddle3 = paddle[3];
        // First vsync may take a while (boot code before display kernel)
        if (sim.run_frame(FIRST_VSYNC_TIMEOUT) < 0) {
            fprintf(stderr, "FATAL: no vsync within %ld clks of reset release\n", FIRST_VSYNC_TIMEOUT);
            return false;
        }
        // 60 NOOP frames (the first vsync above counts as frame 1's boundary;
        // run 59 more to complete 60 frame-periods post-reset, matching ALE's
        // emulate(NOOP, 60) which runs 60 whole frames)
        top->joy0 = action_to_joy(0, false); top->joy1 = 0x7f;
        for (int i = 0; i < 59; i++)
            if (sim.run_frame(FRAME_TIMEOUT) < 0) { fprintf(stderr, "FATAL: vsync lost in noop boot (frame %d)\n", i); return false; }
        // two softResets: 4+4 frames with console RESET switch held
        top->joy0 = action_to_joy(0, true);
        for (int i = 0; i < 8; i++)
            if (sim.run_frame(FRAME_TIMEOUT) < 0) { fprintf(stderr, "FATAL: vsync lost in softReset (frame %d)\n", i); return false; }
        top->joy0 = action_to_joy(0, false);
        rew = RewardState{}; rew.mode = reward_mode(reward_name);
        // prime reward baseline on post-boot state
        uint8_t ram[128]; sim.read_ram(ram); rew.step(ram);
        return true;
    };

    if (boot == "ale") {
        if (!boot_ale()) return 2;
    } else {
        sim.hard_reset_and_zero_ram();
        if (sim.run_frame(FIRST_VSYNC_TIMEOUT) < 0) { fprintf(stderr, "FATAL: no vsync after reset\n"); return 2; }
    }

    if (interactive) {
        setvbuf(stdout, nullptr, _IOLBF, 0);
        printf("READY 128\n");
        char line[256];
        long step_idx = 0;
        while (fgets(line, sizeof line, stdin)) {
            if (line[0] == 'q') break;
            if (line[0] == 'r') {
                if (!boot_ale()) return 2;
                step_idx = 0;
                printf("READY 128\n");
                continue;
            }
            if (line[0] == 's') {
                int action = atoi(line + 1);
                top->joy0 = action_to_joy(action, false);
                long c = sim.run_frame(FRAME_TIMEOUT);
                if (c < 0) { printf("ERR vsync-timeout\n"); continue; }
                uint8_t ram[128]; sim.read_ram(ram);
                int r = rew.step(ram);
                char hex[257]; hexdump_ram(ram, hex);
                printf("f %ld %d %s\n", step_idx++, r, hex);
                continue;
            }
            printf("ERR unknown-cmd\n");
        }
        sim.report_hotspots();
        delete top;
        return 0;
    }

    // batch mode
    std::vector<int> actions;
    if (!script_path.empty()) {
        FILE* sf = fopen(script_path.c_str(), "r");
        if (!sf) { fprintf(stderr, "FATAL: cannot open script %s\n", script_path.c_str()); return 2; }
        char line[64];
        while (fgets(line, sizeof line, sf)) {
            bool full = strchr(line, '\n') != nullptr;
            if (line[0] != '#' && line[0] != '\n') actions.push_back(atoi(line));
            // Consume the remainder of an over-long line so its tail never
            // parses as a fresh record. (caught live: a 73-char search-tool
            // comment split at the 64-byte buffer; the tail "ated=..."
            // atoi'd to a spurious NOOP frame — witnessed as frames=707 vs
            // ALE's 706 in the differential, nothing else.)
            while (!full && fgets(line, sizeof line, sf))
                full = strchr(line, '\n') != nullptr;
        }
        fclose(sf);
    }
    long n = (max_frames > 0) ? max_frames : (long)actions.size();
    if (n == 0) { fprintf(stderr, "FATAL: no frames requested (need --script or --frames)\n"); return 2; }

    FILE* tf = nullptr;
    if (!trace_path.empty()) {
        tf = fopen(trace_path.c_str(), "wb");
        if (!tf) { fprintf(stderr, "FATAL: cannot open trace %s\n", trace_path.c_str()); return 2; }
        write_trace_header(tf);
    }
    FILE* ff = nullptr;
    if (!framedump_path.empty()) {
        ff = fopen(framedump_path.c_str(), "wb");
        if (!ff) { fprintf(stderr, "FATAL: cannot open framedump %s\n", framedump_path.c_str()); return 2; }
        write_afrm_header(ff);
    }

    long total_clks = 0;
    for (long i = 0; i < n; i++) {
        int action = (i < (long)actions.size()) ? actions[i] : 0;
        top->joy0 = action_to_joy(action, false);
        long c = sim.run_frame(FRAME_TIMEOUT);
        if (c < 0) { fprintf(stderr, "FATAL: vsync timeout at frame %ld\n", i); return 3; }
        total_clks += c;
        uint8_t ram[128]; sim.read_ram(ram);
        int32_t r = rew.step(ram);
        if (tf) { fwrite(ram, 1, 128, tf); fwrite(&r, 4, 1, tf); }
        if (ff) fwrite(sim.fbuf, 1, sizeof sim.fbuf, ff);
    }
    if (tf) fclose(tf);
    if (ff) fclose(ff);
    fprintf(stderr, "OK frames=%ld clks=%ld clks/frame=%.1f\n", n, total_clks, (double)total_clks / n);
    sim.report_hotspots();
    delete top;
    return 0;
}
