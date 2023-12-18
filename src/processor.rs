use std::{thread, time};

use crate::{drivers::DisplayDriver, font::FONT_SET, CHIP8_MEMORY};

const CHIP8_PROGRAM_MEMORY_START: usize = 0x200;
pub struct Processor {
    ram: [u8; CHIP8_MEMORY],
    display: [[u8; 64]; 32],
    stack: [usize; 16],      // size of stack hardly even matters
    var_registers: [u8; 16], // general purpose variable registers -- V0 -> VF. VF also used as flag register
    pc: usize,               // Program counter
    index_register: usize,   // Point at locations
    sp: usize,               // Point at next element in the stack
    sound_timer: u8,         // Used to produce beep if value >0
    delay_timer: u8,         // Decremented 60 times per second until it reaches 0
    display_driver: DisplayDriver,
}

impl Processor {
    pub fn new(disp: DisplayDriver) -> Self {
        let mut ram = [0u8; CHIP8_MEMORY];

        // Load the font into memory.
        for i in 0..FONT_SET.len() {
            ram[i] = FONT_SET[i];
        }

        Processor {
            ram: ram,
            sound_timer: 0,
            delay_timer: 0,
            stack: [0; 16],
            display: [[0; 64]; 32],
            var_registers: [0; 16],
            index_register: 0,
            pc: CHIP8_PROGRAM_MEMORY_START, // Program counter starts at 0x200 because 0x000-0x1FF stores the font
            sp: 0,
            display_driver: disp,
        }
    }

    pub fn load_program(&mut self, prog_data: &[u8]) {
        for (i, &byte) in prog_data.iter().enumerate() {
            let address = CHIP8_PROGRAM_MEMORY_START + i;

            if address >= CHIP8_MEMORY {
                panic!("Program data too large to load in memory")
            }

            self.ram[address] = byte;
        }

        println!("Successfully loaded program into memory")
    }

    pub fn start(&mut self) {
        // Sleep for 5 milliseconds

        let sleep_duration = time::Duration::from_millis(5);
        loop {
            let instruction = self.get_instruction();

            self.decode_and_execute_instruction(instruction);
            thread::sleep(sleep_duration)
        }
    }

    fn push_addr(&mut self, addr: usize) {
        self.stack[self.sp] = addr;
        self.sp += 1;
    }

    fn pop_addr(&mut self) -> usize {
        self.sp -= 1;

        self.stack[self.sp]
    }

    // Fetches instruction, which is 2 successive bytes in memory. Increments the program counter by 2 (to be ready for next instruction)
    fn get_instruction(&mut self) -> u16 {
        let instruction: u16 = (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16;

        self.pc += 2;
        return instruction;
    }

    fn decode_and_execute_instruction(&mut self, instruction: u16) {
        let nibbles: (u16, u16, u16, u8) = (
            (instruction & 0xF000) >> 12 as u8,
            (instruction & 0x0F00) >> 8 as u8,
            (instruction & 0x00F0) >> 4 as u8,
            (instruction & 0x000F) as u8,
        );

        let nnn = (instruction & 0x0FFF) as usize; // Last 3 nibbles
        let nn: u8 = (instruction & 0x00FF) as u8; // Last byte
        let x: usize = nibbles.1 as usize; // Second nibble
        let y: usize = nibbles.2 as usize; // Third nibble
        let n: usize = nibbles.3 as usize; // Fourth nibble

        match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => {
                // Clear screen
                self.instruction_clear_screen();
            }
            (0x00, 0x00, 0x0e, 0x0e) => {
                // Return from subroutine
                self.instruction_return();
            }
            (0x06, _, _, _) => {
                // Set variable address value
                self.instruction_set(x, nn);
            }
            (0x07, _, _, _) => {
                // Add to variable address
                self.instruction_add(x, nn);
            }
            (0x01, _, _, _) => {
                // Jump to address
                self.instruction_jmp(nnn);
            }
            (0x02, _, _, _) => {
                // Call Subroutine
                self.instruction_call_subroutine(nnn);
            }
            (0x0A, _, _, _) => {
                // Set index register
                self.instruction_set_index(nnn);
            }
            (0x0D, _, _, _) => {
                // Display and Draw
                self.instruction_draw_display(x, y, n);
                println!("drawing on display!");
            }
            _ => println!("Not supported yet!"),
        }
    }

    fn instruction_jmp(&mut self, addr: usize) {
        self.pc = addr;
    }

    fn instruction_clear_screen(&mut self) {
        for i in 0..self.display.len() {
            for j in 0..self.display[i].len() {
                self.display[i][j] = 0;
            }
        }
    }

    fn instruction_draw_display(&mut self, vx: usize, vy: usize, height: usize) {
        let row = self.var_registers[vy] as usize;
        let col = self.var_registers[vx] as usize;

        self.var_registers[0xF] = 0;

        for i in 0..height {
            let sprite_row = self.ram[self.index_register + i];

            // For each bit in the orw
            for j in 0..8 {
                let bit = (sprite_row >> j) & 1;

                let pixel_screen = self.display[(row + i) % 32][(col + 7 - j) % 64];

                if bit == 1 && pixel_screen == 1 {
                    // We're going to unset a pixel, so set flag in VF
                    self.var_registers[0xF] = 1;
                }

                self.display[(row + i) % 32][(col + 7 - j) % 64] ^= bit;
            }
        }

        self.display_driver.draw(&self.display);
    }

    fn instruction_call_subroutine(&mut self, addr: usize) {
        self.push_addr(self.pc);
        self.pc = addr;
    }

    fn instruction_return(&mut self) {
        let ret_address = self.pop_addr();
        self.pc = ret_address;
    }

    fn instruction_set(&mut self, register: usize, value: u8) {
        self.var_registers[register] = value;
    }

    fn instruction_add(&mut self, register: usize, value: u8) {
        self.var_registers[register] += value
    }

    fn instruction_set_index(&mut self, value: usize) {
        self.index_register = value;
    }
}
