// E0 (Parker Brothers 8K) cartridge mapper — wrapper-side banking
// logic shared by sim_top.v and synth_top.v. The vendored console is
// untouched: the real PB5760 cart carried this logic in its own glue chip,
// so the mapper is the cartridge's half of the bus contract, which was
// always the wrapper's to supply.
//
// E0 semantics (mirroring Stella's CartE0.cxx): 8K = eight 1K banks; the
// 4K window is four 1K slices; slices 0-2 select a bank by ANY valid CPU
// access (read or write) to $1FE0+b / $1FE8+b / $1FF0+b; slice 3
// ($xC00-$xFFF) is hard-wired to bank 7 (vectors + hotspot addresses).
//
// - `rom_read` (atari2600.v:201, already a console port) = cpu_enable &&
//   !stall_cpu && rom_cs: it supplies the A12 fact (without it, TIA/RIOT
//   accesses would alias into false hotspot hits) and strobes once per CPU
//   access, matching Stella's one-checkSwitchBank-per-access.
// - Same-cycle ordering is unobservable by construction: every hotspot
//   lies inside fixed slice 3, so the triggering access itself fetches
//   bank-7 data regardless of the pending register update; only the NEXT
//   access can see a switched slice, and the registered update lands well
//   before the next cpu_enable strobe.
// - Reset loads Stella's non-random parity banks (4,5,6) — the same
//   semantics as CartE0::reset(); AIG latch init stays uniformly 0 and the
//   boot protocol's reset hold does the loading on both sides.
`default_nettype none

module mapper_e0 (
  input  wire        clk,
  input  wire        reset,        // console reset: slices load banks 4,5,6
  input  wire        rom_read,     // valid CPU access to cart space
  input  wire [11:0] rom_address,  // offset within the 4K window
  output wire [12:0] rom_index     // into the 8K image: {bank, addr[9:0]}
);

  reg [2:0] slice0, slice1, slice2;   // +9 flip-flops; slice 3 fixed = 7

  always @(posedge clk) begin
    if (reset) begin
      slice0 <= 3'd4; slice1 <= 3'd5; slice2 <= 3'd6;
    end else if (rom_read) begin
      if      ((rom_address & 12'hFF8) == 12'hFE0) slice0 <= rom_address[2:0];
      else if ((rom_address & 12'hFF8) == 12'hFE8) slice1 <= rom_address[2:0];
      else if ((rom_address & 12'hFF8) == 12'hFF0) slice2 <= rom_address[2:0];
      // $FF8-$FFF (6502 vectors) never switch.
    end
  end

  wire [2:0] bank = (rom_address[11:10] == 2'b00) ? slice0
                  : (rom_address[11:10] == 2'b01) ? slice1
                  : (rom_address[11:10] == 2'b10) ? slice2
                  : 3'd7;

  assign rom_index = {bank, rom_address[9:0]};

endmodule
