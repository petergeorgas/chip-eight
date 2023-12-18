use sdl2::event;
use sdl2::keyboard::Scancode;

pub struct InputDriver {
    event_pump: sdl2::EventPump,
}

impl InputDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_pump = sdl_context.event_pump().unwrap();

        InputDriver { event_pump }
    }

    /*
       Gets the last input to the program. If the last input was an escape key or quit event, the program will exit.
       Otherwise, it looks to see the last key key pressed and attempt to map it to the corresponding CHIP-8 keycode.
    */
    pub fn last_input(&mut self) -> Option<u8> {
        let last_event = match self.event_pump.poll_iter().last() {
            Some(event) => event,
            _ => return None,
        };

        match last_event {
            event::Event::Quit { .. }
            | event::Event::KeyDown {
                scancode: Some(Scancode::Escape),
                ..
            } => {
                println!("Exiting...");
                std::process::exit(1);
            }

            event::Event::KeyDown { .. } => {
                let last_key: Scancode = last_event.as_user_event_type().unwrap();

                // Filter to only keys we care about
                return convert_std_to_chip8_code(last_key);
            }

            _ => return None,
        }
    }
}

/*
    Attempts to map the keyboard keycode to the corresponding CHIP-8 Keycode.
*/
fn convert_std_to_chip8_code(code: Scancode) -> Option<u8> {
    return match code {
        // First row 123C
        Scancode::Num1 => Some(0x1),
        Scancode::Num2 => Some(0x2),
        Scancode::Num3 => Some(0x3),
        Scancode::Num4 => Some(0xC),

        // Second row 456D
        Scancode::Q => Some(0x4),
        Scancode::W => Some(0x5),
        Scancode::E => Some(0x6),
        Scancode::R => Some(0xD),

        // Third row 789E
        Scancode::A => Some(0x7),
        Scancode::S => Some(0x8),
        Scancode::D => Some(0x9),
        Scancode::F => Some(0xE),

        // Fourth row A0BF
        Scancode::Z => Some(0xA),
        Scancode::X => Some(0x0),
        Scancode::C => Some(0xB),
        Scancode::V => Some(0xF),
        _ => None,
    };
}
