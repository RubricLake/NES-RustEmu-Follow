const FLAG_CARRY:            u8 = 0b0000_0001; // bit 0
const FLAG_ZERO:             u8 = 0b0000_0010; // bit 1
const FLAG_INTERRUPT_DISABLE:u8 = 0b0000_0100; // bit 2
const FLAG_DECIMAL_MODE:     u8 = 0b0000_1000; // bit 3
const FLAG_BREAK:            u8 = 0b0001_0000; // bit 4
const FLAG_UNUSED:           u8 = 0b0010_0000; // bit 5 (should always read as 1 on NES)
const FLAG_OVERFLOW:         u8 = 0b0100_0000; // bit 6
const FLAG_NEGATIVE:         u8 = 0b1000_0000; // bit 7





pub struct CPU {
    pub register_a: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            status: 0,
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opcode = program[self.program_counter as usize];
            self.program_counter += 1;

            match opcode {
                0xA9 => {
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.register_a = param;

                    if self.register_a == 0 {
                        self.status |= FLAG_ZERO;
                    } else {
                        self.status &= !FLAG_ZERO;
                    }

                    if self.register_a & 0b1000_0000 != 0 {
                        self.status |= FLAG_NEGATIVE;
                    } else {
                        self.status &= !FLAG_NEGATIVE;
                    }
                }
                0x00 => {
                    return;
                }

                _ => todo!("")
            }
        }
    }

    // Returns true if the given flag is set.
    fn check(&self, flag: u8) -> bool {
        return self.status & flag != 0; 
    }
}


// CPU Testing Here
#[cfg(test)]
mod test {
    use super::*;

    // LDA 
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xA9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.check(FLAG_ZERO));
        assert!(!cpu.check(FLAG_NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.check(FLAG_ZERO));
    }

    #[test] fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0b1000_0001, 0x00]);
        assert!(cpu.check(FLAG_NEGATIVE));
    }
}