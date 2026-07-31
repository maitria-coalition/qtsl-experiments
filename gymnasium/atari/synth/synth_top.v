// synthesis top for the CLOSED per-game circuit env
// (console + cartridge ROM baked in; the env's full transition relation).
// ROM image path comes in as a macro: yosys read_verilog -DROM_HEX="path".
//
// Build variants:
//   -DMAPPER_E0   E0 banked 8K cart via ../sim/mapper_e0.v (montezuma);
//                 default = plain 4K cart.
//   -DVIDEO_OUT   expose the TIA video[6:0] pixel stream as an AIG output
//                 (the pixel-witness lane). ON for all three carts'
//                 emission lane since 2026-07-20;
//                 the certs/ rows were re-certified against the VIDEO_OUT
//                 pins in that same emission. DECODE FACT, preserve in every
//                 pixel consumer: video is REGISTERED in the TIA — the
//                 pixel stream lags the (xpos,ypos) counters by one clk
//                 (pixel at counter (x,y) appears on video[6:0] one clock
//                 later).
`default_nettype none

module synth_top (
  input  wire        clk,
  input  wire        reset,
  input  wire [3:0]  switches,
  input  wire [6:0]  joy0,
  input  wire [6:0]  joy1,
  input  wire [7:0]  paddle0,
  input  wire [7:0]  paddle1,
  input  wire [7:0]  paddle2,
  input  wire [7:0]  paddle3,
  output wire        vsync,
  output wire        vblank,
`ifdef VIDEO_OUT
  output wire [6:0]  video,
`endif
  output wire [7:0]  xpos,
  output wire [8:0]  ypos
);

`ifdef MAPPER_E0
  reg [7:0] rom [0:8191];
`else
  reg [7:0] rom [0:4095];
`endif
  initial $readmemh(`ROM_HEX, rom);

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

`ifndef VIDEO_OUT
  wire [6:0] video;
`endif
  wire [4:0] audio;
`ifdef VIDEO_OUT
  wire _unused = &{rom_read, rom_cycle, audio};
`else
  wire _unused = &{rom_read, rom_cycle, video, audio};
`endif

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
    .audio(audio)
  );

endmodule
