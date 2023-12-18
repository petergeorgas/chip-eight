use std::{thread, time};

use crate::{
    drivers::{DisplayDriver, InputDriver},
    font::FONT_SET,
    CHIP8_MEMORY,
};

const CHIP8_PROGRAM_MEMORY_START: usize = 0x200;
const CHIP8_VF_INDEX: usize = 0x0F;
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
    input_driver: InputDriver,
}

impl Processor {
    pub fn new(disp: DisplayDriver, input: InputDriver) -> Self {
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
            input_driver: input,
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
            // Look for quit event
            let input_key_code = self.input_driver.last_input();

            let instruction = self.get_instruction();

            self.decode_and_execute_instruction(instruction, input_key_code);
            thread::sleep(sleep_duration)
        }
    }

    fn push_addr(&mut self, address: usize) {
        self.stack[self.sp] = address;
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

    fn decode_and_execute_instruction(&mut self, instruction: u16, keycode: Option<u8>) {
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
            (0x0B, _, _, _) => self.instruction_jump_with_offset(nnn),
            (0x0C, _, _, _) => self.instruction_random(x, nn),
            (0x0D, _, _, _) => {
                // Display and Draw
                self.instruction_draw_display(x, y, n);
            }
            (0x03, _, _, _) => {
                // Skip one instruction if VX == NN
                self.instruction_skip_equal(x, nn)
            }
            (0x0E, _, 0x09, 0x0E) => {
                self.instruction_skip_key(x, keycode.unwrap());
            }
            (0x0E, _, 0x0A, 0x01) => {
                self.instruction_skip_not_key(x, keycode.unwrap());
            }
            (0x04, _, _, _) => self.instruction_skip_not_equal(x, nn),
            (0x05, _, _, 0x00) => self.instruction_skip_register_equal(x, y),
            (0x09, _, _, 0x00) => self.instruction_skip_register_not_equal(x, y),
            (0x08, _, _, 0x00) => self.instruction_alu_set(x, y),
            (0x08, _, _, 0x01) => self.instruction_alu_or(x, y),
            (0x08, _, _, 0x02) => self.instruction_alu_and(x, y),
            (0x08, _, _, 0x03) => self.instruction_alu_xor(x, y),
            (0x08, _, _, 0x04) => self.instruction_alu_add(x, y),
            (0x08, _, _, 0x05) => self.instruction_alu_subtract(x, y),
            (0x08, _, _, 0x07) => self.instruction_alu_subtract(y, x),
            (0x08, _, _, 0x06) => self.instruction_alu_shift(x, y, false),
            (0x08, _, _, 0x0E) => self.instruction_alu_shift(x, y, true),

            _ => println!("0x{:04x} Not supported yet!", instruction),
        }
    }

    fn instruction_jmp(&mut self, address: usize) {
        self.pc = address;
    }

    fn instruction_jump_with_offset(&mut self, address: usize) {
        // TODO: AMBIGUOUS INSTRUCTION -- ADD CONFIG FOR THIS TO SUPPORT CHIP-48/SUPER-CHIP
        self.instruction_jmp(address + self.var_registers[0x00] as usize);
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

        self.var_registers[CHIP8_VF_INDEX] = 0;

        for i in 0..height {
            let sprite_row = self.ram[self.index_register + i];

            // For each bit in the orw
            for j in 0..8 {
                let bit = (sprite_row >> j) & 1;

                let pixel_screen = self.display[(row + i) % 32][(col + 7 - j) % 64];

                if bit == 1 && pixel_screen == 1 {
                    // We're going to unset a pixel, so set flag in VF
                    self.var_registers[CHIP8_VF_INDEX] = 1;
                }

                self.display[(row + i) % 32][(col + 7 - j) % 64] ^= bit;
            }
        }

        self.display_driver.draw(&self.display);
    }

    fn instruction_call_subroutine(&mut self, address: usize) {
        self.push_addr(self.pc);
        self.pc = address;
    }

    fn instruction_return(&mut self) {
        let ret_address = self.pop_addr();
        self.pc = ret_address;
    }

    fn instruction_set(&mut self, register: usize, value: u8) {
        self.var_registers[register] = value;
    }

    fn instruction_add(&mut self, register: usize, value: u8) {
        // ADD WITHOUT CARRY FLAG

        let (value, _) = self.var_registers[register].overflowing_add(value);
        self.var_registers[register] = value;
    }

    fn instruction_set_index(&mut self, value: usize) {
        self.index_register = value;
    }

    fn instruction_skip_equal(&mut self, register: usize, value: u8) {
        if self.var_registers[register] == value {
            self.pc += 2
        }
    }

    fn instruction_skip_not_equal(&mut self, register: usize, value: u8) {
        if self.var_registers[register] != value {
            self.pc += 2
        }
    }

    fn instruction_skip_key(&mut self, register: usize, keycode: u8) {
        if self.var_registers[register] == keycode {
            self.pc += 2
        }
    }

    fn instruction_skip_not_key(&mut self, register: usize, keycode: u8) {
        if self.var_registers[register] != keycode {
            self.pc += 2
        }
    }

    fn instruction_skip_register_equal(&mut self, vx_register: usize, vy_register: usize) {
        if self.var_registers[vx_register] == self.var_registers[vy_register] {
            self.pc += 2;
        }
    }

    fn instruction_skip_register_not_equal(&mut self, vx_register: usize, vy_register: usize) {
        if self.var_registers[vx_register] != self.var_registers[vy_register] {
            self.pc += 2;
        }
    }

    fn instruction_random(&mut self, vx_register: usize, value: u8) {
        // Randomly generates a number, ANDs it with value, and stores it in vx register
        let random_value = rand::random::<u8>();

        self.var_registers[vx_register] = random_value & value
    }

    fn instruction_alu_set(&mut self, vx_register: usize, vy_register: usize) {
        // Set value of vx register to value of vy register
        self.var_registers[vx_register] = self.var_registers[vy_register]
    }

    fn instruction_alu_or(&mut self, vx_register: usize, vy_register: usize) {
        // Binary OR
        // Or vx register value with vy register value and store in vx register
        self.var_registers[vx_register] |= self.var_registers[vy_register]
    }

    fn instruction_alu_and(&mut self, vx_register: usize, vy_register: usize) {
        // Binary AND
        // And vx register value with vy register value and store in vx register
        self.var_registers[vx_register] &= self.var_registers[vy_register]
    }

    fn instruction_alu_xor(&mut self, vx_register: usize, vy_register: usize) {
        // Logical XOR
        // Xor vx register value with vy register value and store in vx register
        self.var_registers[vx_register] ^= self.var_registers[vy_register]
    }

    fn instruction_alu_add(&mut self, vx_register: usize, vy_register: usize) {
        // Add vx register value with vy register value and store in vx register
        // if overflow occurred, set VF to 1, else set it to 0
        let (value, overflow) =
            self.var_registers[vx_register].overflowing_add(self.var_registers[vy_register]);

        self.var_registers[vx_register] = value;
        if overflow {
            self.var_registers[CHIP8_VF_INDEX] = 1;
        } else {
            self.var_registers[CHIP8_VF_INDEX] = 0;
        }
    }

    fn instruction_alu_subtract(&mut self, vx_register: usize, vy_register: usize) {
        let (vx_value, vy_value) = (
            self.var_registers[vx_register],
            self.var_registers[vy_register],
        );

        if vx_value > vy_value {
            self.var_registers[CHIP8_VF_INDEX] = 1; // Set VF before subtraction
        }

        let (value, underflow) = vx_value.overflowing_sub(vy_value);
        self.var_registers[vx_register] = value;
        if underflow {
            self.var_registers[CHIP8_VF_INDEX] = 0;
        }
    }

    fn instruction_alu_shift(&mut self, vx_register: usize, vy_register: usize, left_shift: bool) {
        //TODO: OPTIONAL_CONFIGURABLE -- SET VX VALUE TO VY VALUE
        self.var_registers[vx_register] = self.var_registers[vy_register];

        let mut vx_value = self.var_registers[vx_register];

        let vf_value: u8;

        if left_shift {
            // left shift
            vf_value = self.var_registers[vx_register] & 0x80;
            vx_value <<= 1;
        } else {
            // right shift
            vf_value = self.var_registers[vx_register] & 0x01;
            vx_value >>= 1;
        }

        self.var_registers[CHIP8_VF_INDEX] = vf_value;
        self.var_registers[vx_register] = vx_value;
    }
}
