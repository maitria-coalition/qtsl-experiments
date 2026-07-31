// simulation top for the AtariCircuit env.
// Instantiates the vendored atari2600 console directly (no TT wrapper, no
// QSPI) with a plain ROM array — the cartridge as it appears to the
// console: an asynchronous byte oracle on the address bus.
// 2KB ROMs are loaded doubled (mirror baked in by the loader).
//
// Build variants: default = plain 4K cart (pong, freeway);
// -DMAPPER_E0 = E0 banked 8K cart via mapper_e0.v (montezuma). Two
// separate sim builds — see Makefile targets.
//
// Posture 1 (the circuit IS the env): the env's transition relation is this
// module minus the debug ports, i.e. atari2600 (+ ROM) under yosys
// write_aiger/write_btor — see ../synth/.
`default_nettype none

module sim_top (
  input  wire        clk,
  input  wire        reset,
  input  wire [3:0]  switches,   // {sw3, sw2(color=1), sw1(~P0diff), sw0(~P1diff)}
  input  wire [6:0]  joy0,       // {RIGHT,LEFT,DOWN,UP,SELECT,FIRE,RESET} active-low, idle=7'h7f
  input  wire [6:0]  joy1,
  input  wire [7:0]  paddle0,
  input  wire [7:0]  paddle1,
  input  wire [7:0]  paddle2,
  input  wire [7:0]  paddle3,
  output wire        vsync,
  output wire        vblank,
  output wire [7:0]  xpos,
  output wire [8:0]  ypos,
  output wire [6:0]  video,      // TIA {color,luma} pixel stream (pixel-witness tap)
  output wire        dbg_rom_read,     // cart-access strobe (hotspot coverage)
  output wire [11:0] dbg_rom_address,
  output wire [1023:0] ram_flat,
  input  wire        ram_we,
  input  wire [6:0]  ram_waddr,
  input  wire [7:0]  ram_wdata
);

`ifdef MAPPER_E0
  reg [7:0] rom [0:8191];
`else
  reg [7:0] rom [0:4095];
`endif
  initial begin : rom_load
    reg [1023:0] rom_hex_path;
    if ($value$plusargs("ROMHEX=%s", rom_hex_path))
      $readmemh(rom_hex_path, rom);
    else begin
      $display("FATAL: +ROMHEX=<path> required");
      $finish;
    end
  end

  wire [11:0] rom_address;
  wire        rom_read;
  wire        rom_cycle;

`ifdef MAPPER_E0
  wire [12:0] rom_index;
  mapper_e0 mapper (
    .clk(clk), .reset(reset), .rom_read(rom_read),
    .rom_address(rom_address), .rom_index(rom_index)
  );
  wire [7:0]  rom_data = rom[rom_index];
`else
  wire [7:0]  rom_data = rom[rom_address];
`endif

  assign dbg_rom_read    = rom_read;
  assign dbg_rom_address = rom_address;

  wire [4:0] audio;
  wire _unused = &{rom_cycle, audio};

  atari2600 console (
    .clk(clk),
    .reset(reset),
    .system_enable(1'b1),
    .input_switches(switches),
    .input_joystick_0(joy0),
    .input_joystick_1(joy1),
    .input_paddle_0(paddle0),
    .input_paddle_1(paddle1),
    .input_paddle_2(paddle2),
    .input_paddle_3(paddle3),
    .rom_read(rom_read),
    .rom_cycle(rom_cycle),
    .rom_address(rom_address),
    .rom_data(rom_data),
    .video(video),
    .xpos(xpos),
    .ypos(ypos),
    .vsync(vsync),
    .vblank(vblank),
    .audio(audio),
    .debug_ram_flat(ram_flat),
    .debug_ram_we(ram_we),
    .debug_ram_waddr(ram_waddr),
    .debug_ram_wdata(ram_wdata)
  );

endmodule
