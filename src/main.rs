pub mod chip_timers;
pub mod instruction;
pub mod renderer;
pub mod rng;

extern crate sdl2;

use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::fs;
use std::time::Duration;

const WDW_SIZE_SCALAR: u32 = 8;
const WDW_WIDTH: u32 = 64;
const WDW_HEIGHT: u32 = 32;

fn main() -> Result<(), String> {
    let mem = ChipMemory::new();
    let mut emu = ChipEmulator::new(mem)?;
    emu.load_rom("roms/7-beep.ch8")?;

    emu.run_loop()?;
    Ok(())
}

struct AudioManager {
    playing: bool,
    device: sdl2::audio::AudioDevice<ChipBeep>,
}
struct ChipBeep {
    phase_inc: f32,
    phase_state: f32,
    volume: f32,
}
impl sdl2::audio::AudioCallback for ChipBeep {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = if self.phase_state <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase_state = (self.phase_state + self.phase_inc) % 1.0;
        }
    }
}

impl AudioManager {
    fn new(audio_subsystem: sdl2::AudioSubsystem) -> Self {
        let device = audio_subsystem
            .open_playback(
                None,
                &sdl2::audio::AudioSpecDesired {
                    freq: Some(44100),
                    channels: Some(1),
                    samples: None,
                },
                |spec| {
                    println!("{}", spec.freq);
                    ChipBeep {
                        phase_inc: 440.0 / spec.freq as f32,
                        phase_state: 0.0,
                        volume: 0.1,
                    }
                },
            )
            .unwrap();
        Self {
            playing: false,
            device,
        }
    }
    pub fn resume(&mut self) {
        self.device.resume();
        self.playing = true;
    }
    pub fn pause(&mut self) {
        self.device.pause();
        self.playing = false;
    }
}

struct ChipEmulator {
    mem: ChipMemory,
    renderer: Renderer,
    sdl_context: sdl2::Sdl,
    input: [bool; 16],
    rng: rng::RandomNumberGenerator,
    audio: AudioManager,
}

impl ChipEmulator {
    fn new(mem: ChipMemory) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "Rust SDL Demo",
                WDW_WIDTH * WDW_SIZE_SCALAR,
                WDW_HEIGHT * WDW_SIZE_SCALAR,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        let renderer = Renderer::new(window)?;
        let input = [false; 16];
        let mut audio = AudioManager::new(sdl_context.audio().unwrap());
        let mut rng = rng::RandomNumberGenerator::new(4);
        rng.seed_with_time();
        // let pump = sdl_context.event_pump()?;
        Ok(Self {
            mem,
            renderer,
            sdl_context,
            input,
            rng,
            audio,
        })
    }

    pub fn run_loop(&mut self) -> Result<(), String> {
        let mut pump = self.sdl_context.event_pump()?;
        let mut current_time = 0u64;
        let mut auto_clk = true;
        let mut last_delay_time = 0u64;
        'running: loop {
            for event in pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => match keycode {
                        Keycode::Escape => break 'running,
                        Keycode::Space => self.chip_clk()?,
                        Keycode::M => auto_clk = !auto_clk,
                        Keycode::Num1 => self.input[0x1] = true,
                        Keycode::Num2 => self.input[0x2] = true,
                        Keycode::Num3 => self.input[0x3] = true,
                        Keycode::Q => self.input[0x4] = true,
                        Keycode::W => self.input[0x5] = true,
                        Keycode::E => self.input[0x6] = true,
                        Keycode::A => self.input[0x7] = true,
                        Keycode::S => self.input[0x8] = true,
                        Keycode::D => self.input[0x9] = true,
                        Keycode::Z => self.input[0xA] = true,
                        Keycode::C => self.input[0xB] = true,
                        Keycode::Num4 => self.input[0xC] = true,
                        Keycode::R => self.input[0xD] = true,
                        Keycode::F => self.input[0xE] = true,
                        Keycode::V => self.input[0xF] = true,
                        Keycode::X => self.input[0x0] = true,
                        Keycode::N => self.renderer.print_debug(),
                        _ => {}
                    },
                    Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => match keycode {
                        Keycode::Num1 => self.input[0x1] = false,
                        Keycode::Num2 => self.input[0x2] = false,
                        Keycode::Num3 => self.input[0x3] = false,
                        Keycode::Q => self.input[0x4] = false,
                        Keycode::W => self.input[0x5] = false,
                        Keycode::E => self.input[0x6] = false,
                        Keycode::A => self.input[0x7] = false,
                        Keycode::S => self.input[0x8] = false,
                        Keycode::D => self.input[0x9] = false,
                        Keycode::Z => self.input[0xA] = false,
                        Keycode::C => self.input[0xB] = false,
                        Keycode::Num4 => self.input[0xC] = false,
                        Keycode::R => self.input[0xD] = false,
                        Keycode::F => self.input[0xE] = false,
                        Keycode::V => self.input[0xF] = false,
                        Keycode::X => self.input[0x0] = false,
                        _ => {}
                    },
                    _ => {}
                }
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 720)); // Renders 180 times
                                                                            // per second
            current_time += 1_000_000_000u64 / 720;
            let delay_delta_time = current_time - last_delay_time;
            // Tick timers sixty times per second
            if delay_delta_time > 1_000_000_000u64 / 60 {
                last_delay_time = current_time;
                self.mem.timers.tick_second();
                if self.audio.playing & (self.mem.timers.sound == 0) {
                    self.audio.pause();
                }
                if (self.mem.timers.sound > 0) & !self.audio.playing {
                    self.audio.resume();
                }
            }

            if auto_clk {
                self.chip_clk()?;
                self.renderer.draw()?;
            }
        }
        Ok(())
    }

    pub fn load_rom(&mut self, rom_path: &str) -> Result<(), String> {
        println!("Reading rom {}", rom_path);
        let cts = fs::read(rom_path).expect("Unable to read rom");
        let mut i = 0;
        for x in cts {
            self.mem.ram[ChipMemory::ROM_STARTING_MEMORY_LOCATION + i] = x;
            i += 1;
        }
        self.mem.pc = ChipMemory::ROM_STARTING_MEMORY_LOCATION as u16;
        Ok(())
    }
    pub fn chip_clk(&mut self) -> Result<(), String> {
        let instruction = self.mem.get_instruction()?;

        let first_nibble = instruction.get_first_nibble();
        let second_nibble = instruction.get_second_nibble();
        let third_nibble = instruction.get_third_nibble();

        let nn = instruction.get_nn();
        let nnn = instruction.get_nnn();

        match first_nibble {
            0x0 => {
                if instruction.val[1] == 0xE0 {
                    self.renderer.clear_display()
                } else if instruction.val[1] == 0xEE {
                    // Return from subroutine
                    self.mem.pc = self.mem.stack[self.mem.stack_ptr.saturating_sub(1)];
                    self.mem.stack_ptr = self.mem.stack_ptr.saturating_sub(1);
                } else {
                    unimplemented!("Machine code language call not implemented!")
                }
            }
            0xA0 => self.mem.i = nnn,
            0x60 => self.mem.registers[second_nibble as usize] = nn,
            0xD0 => {
                let x_draw_coord = self.mem.registers[second_nibble as usize] % WDW_WIDTH as u8;
                let y_draw_coord = self.mem.registers[third_nibble as usize] % WDW_HEIGHT as u8;
                let sprite_height = instruction.val[1] & 0x0F;
                self.mem.registers[0xF] = 0x0;
                for yi in 0..sprite_height {
                    let sprite_data = self.mem.ram[self.mem.i as usize + yi as usize];
                    // println!("{:#010b}", sprite_data);
                    if (yi + y_draw_coord) as u32 > WDW_HEIGHT {
                        break;
                    }
                    for xi in 0..8u8 {
                        if (xi + x_draw_coord) as u32 > WDW_WIDTH {
                            break;
                        }
                        let pixel_data =
                            ((sprite_data << (xi) as u32) & 0b1000_0000) == 0b1000_0000;
                        let cur_val = self.renderer.get_display_at_location(
                            (xi + x_draw_coord) as usize,
                            (yi + y_draw_coord) as usize,
                        )?;

                        // let ret_val: bool;
                        // if pixel_data & cur_val {
                        //     ret_val = false;
                        //     self.mem.registers[0xF] = 0x1;
                        // } else if pixel_data {
                        //     ret_val = true;
                        // } else {
                        //     ret_val = false;
                        // }

                        // Ohhhh.. you don't make any changes if pixel_data is false....
                        // Me big dum
                        let ret_val: bool;
                        if !pixel_data {
                            continue;
                        } else if cur_val {
                            ret_val = false;
                            self.mem.registers[0xF] = 0x1;
                        } else {
                            ret_val = true;
                        }

                        self.renderer.set_display_at_location(
                            (xi + x_draw_coord) as usize,
                            (yi + y_draw_coord) as usize,
                            ret_val,
                        )?;
                    }
                }
            }
            0x70 => {
                (self.mem.registers[second_nibble as usize], _) =
                    self.mem.registers[second_nibble as usize].overflowing_add(nn)
            }
            0x10 => self.mem.pc = nnn,
            0x20 => {
                self.mem.stack[self.mem.stack_ptr] = self.mem.pc;
                self.mem.stack_ptr += 1;
                self.mem.pc = nnn
            }
            0x30 => {
                let x = self.mem.registers[second_nibble as usize];
                if x == nn {
                    self.mem.pc += 2
                }
            }
            0x40 => {
                let x = self.mem.registers[second_nibble as usize];
                if x != nn {
                    self.mem.pc += 2;
                }
            }
            0x50 => {
                let x = self.mem.registers[second_nibble as usize];
                let y = self.mem.registers[third_nibble as usize];
                if x == y {
                    self.mem.pc += 2;
                }
            }
            0x90 => {
                let x = self.mem.registers[second_nibble as usize];
                let y = self.mem.registers[third_nibble as usize];
                if x != y {
                    self.mem.pc += 2;
                }
            }
            0xB0 => {
                let reg_zero = self.mem.registers[0];
                self.mem.pc = nnn + reg_zero as u16
            }
            0xC0 => {
                self.mem.registers[second_nibble as usize] = nn & self.rng.next();
            }
            0x80 => match instruction.val[1] & 0x0F {
                0x0 => {
                    self.mem.registers[second_nibble as usize] =
                        self.mem.registers[third_nibble as usize]
                }
                0x1 => {
                    self.mem.registers[second_nibble as usize] = self.mem.registers
                        [second_nibble as usize]
                        | self.mem.registers[third_nibble as usize]
                }
                0x2 => {
                    self.mem.registers[second_nibble as usize] = self.mem.registers
                        [second_nibble as usize]
                        & self.mem.registers[third_nibble as usize]
                }
                0x3 => {
                    self.mem.registers[second_nibble as usize] = self.mem.registers
                        [second_nibble as usize]
                        ^ self.mem.registers[third_nibble as usize]
                }
                0x4 => {
                    let x = self.mem.registers[second_nibble as usize];
                    let y = self.mem.registers[third_nibble as usize];
                    let (ret_val, overflow) = x.overflowing_add(y);
                    self.mem.registers[second_nibble as usize] = ret_val;
                    self.mem.registers[0xF] = if overflow { 0x1 } else { 0x0 };
                }
                0x5 => {
                    let x = self.mem.registers[second_nibble as usize];
                    let y = self.mem.registers[third_nibble as usize];

                    let (ret_val, _) = self.mem.registers[second_nibble as usize]
                        .overflowing_sub(self.mem.registers[third_nibble as usize]);
                    self.mem.registers[second_nibble as usize] = ret_val;
                    self.mem.registers[0xF] = if x >= y { 0x1 } else { 0x0 };
                }
                0x7 => {
                    let x = self.mem.registers[second_nibble as usize];
                    let y = self.mem.registers[third_nibble as usize];
                    let (ret_val, _) = y.overflowing_sub(x);
                    self.mem.registers[second_nibble as usize] = ret_val;
                    self.mem.registers[0xF] = if y >= x { 0x1 } else { 0x0 };
                }
                0x6 => {
                    self.mem.registers[second_nibble as usize] =
                        self.mem.registers[third_nibble as usize];
                    let orig = self.mem.registers[second_nibble as usize];
                    self.mem.registers[second_nibble as usize] =
                        self.mem.registers[second_nibble as usize] >> 1;
                    self.mem.registers[0xF] = if (orig & 0b0000_0001) == 1 { 0x1 } else { 0x0 };
                }
                0xE => {
                    self.mem.registers[second_nibble as usize] =
                        self.mem.registers[third_nibble as usize];
                    let orig = self.mem.registers[second_nibble as usize];
                    self.mem.registers[second_nibble as usize] =
                        self.mem.registers[second_nibble as usize] << 1;
                    self.mem.registers[0xF] = if (orig & 0b1000_0000) == 0 { 0x0 } else { 0x1 };
                }
                _ => todo!("Unimplemented opcode: {:#04x?}", instruction),
            },
            0xE0 => match instruction.val[1] {
                0x9E => {
                    // skip if key is pressed
                    let which_key = self.mem.registers[second_nibble as usize];
                    if self.input[which_key as usize] {
                        self.mem.pc += 2;
                    }
                }
                0xA1 => {
                    // skip if key is not pressed
                    let which_key = self.mem.registers[second_nibble as usize];
                    if !self.input[which_key as usize] {
                        self.mem.pc += 2;
                    }
                }
                _ => todo!("Unimplemented opcode: {:#04x?}", instruction),
            },
            0xF0 => match instruction.val[1] {
                0x07 => self.mem.registers[second_nibble as usize] = self.mem.timers.delay,
                0x15 => self.mem.timers.delay = self.mem.registers[second_nibble as usize],
                0x18 => self.mem.timers.sound = self.mem.registers[second_nibble as usize],
                0x1E => {
                    let (o, _) = self
                        .mem
                        .i
                        .overflowing_add(self.mem.registers[second_nibble as usize] as u16);
                    self.mem.i = o;
                }
                0x55 => {
                    for i in 0..=second_nibble as usize {
                        self.mem.ram[self.mem.i as usize + i] = self.mem.registers[i]
                    }
                }
                0x65 => {
                    for i in 0..=second_nibble as usize {
                        self.mem.registers[i] = self.mem.ram[self.mem.i as usize + i]
                    }
                }
                0x33 => {
                    let x = self.mem.registers[second_nibble as usize];
                    let x1 = x / 100;
                    let x2 = (x % 100) / 10;
                    let x3 = x % 10;
                    self.mem.ram[self.mem.i as usize] = x1;
                    self.mem.ram[(self.mem.i + 1u16) as usize] = x2;
                    self.mem.ram[(self.mem.i + 2u16) as usize] = x3;
                }
                0x0A => {
                    let mut pressed_key = 0u8;
                    let mut no_pressed_key = true;
                    'get_key_loop: for i in 0..16u8 {
                        if self.input[i as usize] {
                            no_pressed_key = false;
                            pressed_key = i;
                            break 'get_key_loop;
                        }
                    }
                    if no_pressed_key {
                        self.mem.pc -= 2;
                    } else {
                        self.mem.registers[second_nibble as usize] = pressed_key;
                    }
                }
                0x29 => {
                    let x = self.mem.registers[second_nibble as usize];
                    let memory_address =
                        ChipMemory::FONT_ROM_STARTING_MEMORY_LOCATION + (5usize * x as usize);
                    self.mem.i = memory_address as u16;
                }
                _ => todo!("Unimplemented opcode: {:#04x?}", instruction),
            },

            _ => unimplemented!("Unimplmeted opcode: {:#04x?}", instruction),
        }

        Ok(())
    }
}

struct ChipMemory {
    ram: [u8; 4096],
    pc: u16,
    i: u16,
    stack: [u16; 32],
    stack_ptr: usize,
    timers: chip_timers::ChipTimers,
    registers: [u8; 16],
}

impl ChipMemory {
    const ROM_STARTING_MEMORY_LOCATION: usize = 0x200;
    const FONT_DATA: [u8; 0x10 * 5usize] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, //0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];
    const FONT_ROM_STARTING_MEMORY_LOCATION: usize = 0x50;
    pub fn new() -> Self {
        let mut ram = [0u8; 4096];
        let mut my_i = Self::FONT_ROM_STARTING_MEMORY_LOCATION;
        for datum in Self::FONT_DATA {
            ram[my_i] = datum;
            my_i += 1;
        }

        Self {
            ram,
            pc: 0u16,
            i: 0u16,
            stack: [0u16; 32],
            stack_ptr: 0,
            timers: chip_timers::ChipTimers::new(),
            registers: [0u8; 16],
        }
    }

    pub fn get_instruction(&mut self) -> Result<instruction::Instruction, String> {
        let k = [self.ram[self.pc as usize], self.ram[(self.pc + 1) as usize]];
        let ret_val = instruction::Instruction::new(k);
        // println!("Instruction loaded: {:02x?}", k);
        self.pc += 2;
        Ok(ret_val)
    }
}
