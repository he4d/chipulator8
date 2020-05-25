extern crate rand;

use rand::Rng;
use std::fs;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

pub struct Chip8 {
    pub key: [u8; 16],
    pub gfx: [u8; 64 * 32],
    pub draw_flag: bool,
    pub sound_timer: u8,
    pc: u16,
    opcode: u16,
    i: u16,
    sp: u16,
    v: [u8; 16],
    stack: [u16; 16],
    memory: [u8; 4096],
    delay_timer: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip = Chip8 {
            key: [0; 16],
            gfx: [0; 2048],
            draw_flag: false,
            pc: 0x200,
            opcode: 0,
            i: 0,
            sp: 0,
            v: [0; 16],
            stack: [0; 16],
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
        };
        for (i, font) in FONTSET.iter().enumerate() {
            chip.memory[i] = *font;
        }
        chip
    }
    pub fn emulate_cycle(&mut self) {
        self.draw_flag = false;

        self.opcode =
            (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;
        match self.opcode & 0xF000 {
            0x0000 => {
                match self.opcode & 0x000F {
                    // 0x00E0: Clears the screen
                    0x0000 => {
                        for g in self.gfx.iter_mut() {
                            *g = 0x0;
                        }
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    // 0x00EE: Returns from subroutine
                    0x000E => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                        self.pc += 2;
                    }
                    _ => println!("Unknown opcode [0x0000]: {:X?}", self.opcode),
                }
            }
            // 0x1NNN: Jumps to address NNN
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            }
            // 0x2NNN: Calls subroutine at NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            // 0x3NNN: Skips the next instruction if VX equals NN
            0x3000 => {
                if self.v[((self.opcode & 0x0F00) >> 8) as usize] == (self.opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // 0x4XNN: Skips the next instruction if VX doesn't equal NN
            0x4000 => {
                if self.v[((self.opcode & 0x0F00) >> 8) as usize] != (self.opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // 0x5XY0: Skips the next instruction if VX equals VY.
            0x5000 => {
                if self.v[((self.opcode & 0x0F00) >> 8) as usize]
                    == self.v[((self.opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // 0x6XNN: Sets VX to NN.
            0x6000 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            // 0x7XNN: Adds NN to VX.
            0x7000 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.v
                    [((self.opcode & 0x0F00) >> 8) as usize]
                    .overflowing_add((self.opcode & 0x00FF) as u8)
                    .0;
                self.pc += 2;
            }
            0x8000 => {
                match self.opcode & 0x000F {
                    // 0x8XY0: Sets VX to the value of VY
                    0x0000 => {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] =
                            self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }
                    // 0x8XY1: Sets VX to "VX OR VY"
                    0x0001 => {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] |=
                            self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }
                    // 0x8XY2: Sets VX to "VX AND VY"
                    0x0002 => {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] &=
                            self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }
                    // 0x8XY3: Sets VX to "VX XOR VY"
                    0x0003 => {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] ^=
                            self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }
                    // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't
                    0x0004 => {
                        if self.v[((self.opcode & 0x00F0) >> 4) as usize]
                            > (0xFF - self.v[((self.opcode & 0x0F00) >> 8) as usize])
                        {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.v
                            [((self.opcode & 0x0F00) >> 8) as usize]
                            .overflowing_add(self.v[((self.opcode & 0x00F0) >> 4) as usize])
                            .0;
                        self.pc += 2;
                    }
                    // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't
                    0x0005 => {
                        if self.v[((self.opcode & 0x00F0) >> 4) as usize]
                            > self.v[((self.opcode & 0x0F00) >> 8) as usize]
                        {
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.v
                            [((self.opcode & 0x0F00) >> 8) as usize]
                            .overflowing_sub(self.v[((self.opcode & 0x00F0) >> 4) as usize])
                            .0;
                        self.pc += 2;
                    }
                    // 0x8XY6: Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift
                    0x0006 => {
                        self.v[0xF] = self.v[((self.opcode & 0x0F00) >> 8) as usize] & 0x1;
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] >>= 1;
                        self.pc += 2;
                    }
                    // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't
                    0x0007 => {
                        if self.v[((self.opcode & 0x0F00) >> 8) as usize]
                            > self.v[((self.opcode & 0x00F0) >> 4) as usize]
                        {
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.v
                            [((self.opcode & 0x00F0) >> 4) as usize]
                            .overflowing_sub(self.v[((self.opcode & 0x0F00) >> 8) as usize])
                            .0;
                        self.pc += 2;
                    }
                    // 0x8XYE: Shifts VX left by one. VF is set to the value of the most significant bit of VX before the shift
                    0x000E => {
                        self.v[0xF] = self.v[((self.opcode & 0x0F00) >> 8) as usize] >> 7;
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] <<= 1;
                        self.pc += 2;
                    }
                    _ => {
                        println!("Unknown opcode [0x8000]: {:X?}", self.opcode);
                    }
                }
            }
            // 0x9XY0: Skips the next instruction if VX doesn't equal VY
            0x9000 => {
                if self.v[((self.opcode & 0x0F00) >> 8) as usize]
                    != self.v[((self.opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // ANNN: Sets I to the address NNN
            0xA000 => {
                self.i = self.opcode & 0x0FFF;
                self.pc += 2;
            }
            // BNNN: Jumps to the address NNN plus V0
            0xB000 => {
                self.pc = (self.opcode & 0x0FFF) + self.v[0] as u16;
            }
            // CXNN: Sets VX to a random number and NN
            0xC000 => {
                let rnd: u8 = rand::thread_rng().gen();
                self.v[((self.opcode & 0x0F00) >> 8) as usize] =
                    (rnd % 0xFF) & (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            // DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
            // Each row of 8 pixels is read as bit-coded starting from memory location I;
            // I value doesn't change after the execution of this instruction.
            // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
            // and to 0 if that doesn't happen
            0xD000 => {
                let x = (self.opcode & 0x0F00) >> 8 as usize;
                let y = (self.opcode & 0x00F0) >> 4 as usize;
                let height = self.opcode & 0x000F;
                let mut pixel: u16;

                self.v[0xF] = 0;
                for yline in 0..height {
                    let y = (self.v[y as usize] + yline as u8) % 32;
                    pixel = self.memory[(self.i + yline) as usize] as u16;
                    for xline in 0..8 {
                        let x = (self.v[x as usize] + xline as u8) % 64;
                        if (pixel & (0x80 >> xline)) != 0 {
                            let i = (x as u16 + (y as u16 * 64)) as usize;
                            self.v[0xF] |= 1 & self.gfx[i];
                            self.gfx[i] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => match self.opcode & 0x00FF {
                // EX9E: Skips the next instruction if the key stored in VX is pressed
                0x009E => {
                    if self.key[self.v[((self.opcode & 0x0F00) >> 8) as usize] as usize] != 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                // EXA1: Skips the next instruction if the key stored in VX isn't pressed
                0x00A1 => {
                    if self.key[self.v[((self.opcode & 0x0F00) >> 8) as usize] as usize] == 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                _ => println!("Unknown opcode [0xE000]: {:X?}", self.opcode),
            },
            0xF000 => {
                match self.opcode & 0x00FF {
                    // FX07: Sets VX to the value of the delay timer
                    0x0007 => {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.delay_timer;
                        self.pc += 2;
                    }
                    // FX0A: A key press is awaited, and then stored in VX
                    0x000A => {
                        let mut key_press = false;

                        for i in 0..16 {
                            if self.key[i] != 0 {
                                self.v[((self.opcode & 0x0F00) >> 8) as usize] = i as u8;
                                key_press = true;
                            }
                        }

                        if !key_press {
                            return;
                        }

                        self.pc += 2;
                    }
                    // FX15: Sets the delay timer to VX
                    0x0015 => {
                        self.delay_timer = self.v[((self.opcode & 0x0F00) >> 8) as usize];
                        self.pc += 2;
                    }
                    // FX18: Sets the sound timer to VX
                    0x0018 => {
                        self.sound_timer = self.v[((self.opcode & 0x0F00) >> 8) as usize];
                        self.pc += 2;
                    }
                    // FX1E: Adds VX to I
                    0x001E => {
                        if self.i + self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16 > 0xFFF {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.i += self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16;
                        self.pc += 2;
                    }
                    // FX29: Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font
                    0x0029 => {
                        self.i = self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16 * 0x5;
                        self.pc += 2;
                    }
                    // FX33: Stores the Binary-coded decimal representation of VX at the addresses I, I plus 1, and I plus 2
                    0x0033 => {
                        self.memory[self.i as usize] =
                            self.v[((self.opcode & 0x0F00) >> 8) as usize] / 100;
                        self.memory[(self.i + 1) as usize] =
                            (self.v[((self.opcode & 0x0F00) >> 8) as usize] / 10) % 10;
                        self.memory[(self.i + 2) as usize] =
                            (self.v[((self.opcode & 0x0F00) >> 8) as usize] % 100) % 10;
                        self.pc += 2;
                    }
                    // FX55: Stores V0 to VX in memory starting at address I
                    0x0055 => {
                        for i in 0..=((self.opcode & 0x0F00) >> 8) {
                            self.memory[(self.i + i) as usize] = self.v[i as usize];
                        }
                        self.pc += 2;
                    }
                    // FX65: Fills V0 to VX with values from memory starting at address I
                    0x0065 => {
                        for i in 0..=((self.opcode & 0x0F00) >> 8) {
                            self.v[i as usize] = self.memory[(self.i + i) as usize];
                        }
                        self.pc += 2;
                    }
                    _ => println!("Unknown opcode [0xF000]: {:X?}", self.opcode),
                }
            }
            _ => println!("Unknown opcode: {:X?}", self.opcode),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn load_application(&mut self, filename: &str) {
        println!("Loading: {}", filename);

        let filesize: u64;
        match fs::metadata(filename) {
            Ok(metadata) => {
                filesize = metadata.len();
                println!("Filesize: {}", filesize);
            }
            Err(_) => {
                panic!("File error");
            }
        }

        match fs::read(filename) {
            Ok(content) => {
                if (4096 - 512) > filesize {
                    for i in 0..filesize {
                        self.memory[i as usize + 512] = content[i as usize];
                    }
                } else {
                    panic!("Error: ROM too big for memory");
                }
            }
            Err(_) => panic!("File error"),
        }
    }
}
