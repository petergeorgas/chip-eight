use std::{fs, io::Read};

// Program cannot be larger than 4096 - 512 bytes (first 512 bytes are reserved for the font)
const CHIP8_MAX_ROM_SIZE: usize = 3584;

pub struct Rom {
    pub data: [u8; CHIP8_MAX_ROM_SIZE],
}

impl Rom {
    // New reads ROM file into bytes and
    pub fn new(filename: &str) -> Self {
        let mut rom_file = fs::File::open(filename).expect("Could not open file");

        let mut buffer = [0u8; CHIP8_MAX_ROM_SIZE];

        // Attempt to fill the buffer
        let bytes_read = match rom_file.read(&mut buffer) {
            Ok(num_bytes) => num_bytes,
            Err(_) => 0,
        };

        println!("Read total of {} bytes from ROM", bytes_read);

        if bytes_read == 0 {
            panic!("Failed to read ROM")
        }

        Rom { data: buffer }
    }
}
