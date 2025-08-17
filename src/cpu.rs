#![allow(dead_code)]

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
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
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
                    self.lda(param);
                }
                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => return,
                _ => todo!("")
            }
        }
    }

    fn lda(&mut self, value: u8) {
       self.register_a = value;
       self.update_zero_and_negative_flags(self.register_a);
   }
 
    fn tax(&mut self) {
       self.register_x = self.register_a;
       self.update_zero_and_negative_flags(self.register_x);
   }

   fn inx(&mut self) {
        self.register_x += 1;
        self.update_zero_and_negative_flags(self.register_x);
   }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status |= FLAG_ZERO;
        } else {
            self.status &= !FLAG_ZERO;
        }

        if result & FLAG_NEGATIVE != 0 {
            self.status |= FLAG_NEGATIVE;
        } else {
            self.status &= !FLAG_NEGATIVE;
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
    use std::vec;

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

    #[test] 
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0b1000_0001, 0x00]);
        assert!(cpu.check(FLAG_NEGATIVE));
        assert!(cpu.check(!FLAG_ZERO));
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.interpret(vec![0xaa, 0x00]);
        
        assert_eq!(cpu.register_a, 10);
    }

    #[test]
    fn test_0xaa_tax_negative_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1000_0001;
        cpu.interpret(vec![0xaa, 0x00]);

        assert!(cpu.check(FLAG_NEGATIVE));
        assert!(!cpu.check(FLAG_ZERO));
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0;
        cpu.interpret(vec![0xaa, 0x00]);

        assert!(cpu.check(FLAG_ZERO));
        assert!(!cpu.check(FLAG_NEGATIVE));
    }

    #[test]
    fn test_0xe8_inx_increment_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 10;
        cpu.interpret(vec![0xe8, 0x00]);

        assert_eq!(cpu.register_x, 11);
    }

    #[test]
    fn test_0xe8_inx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.register_x = 0b1111_1111;
        cpu.interpret(vec![0xe8, 0x00]);

        assert!(cpu.check(FLAG_ZERO));
        assert!(!cpu.check(FLAG_NEGATIVE))
    }

    #[test]
    fn test_0xe8_inx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.register_x = 0b1000_0001;
        cpu.interpret(vec![0xe8, 0x00]);

        assert!(cpu.check(FLAG_NEGATIVE));
        assert!(!cpu.check(FLAG_ZERO));
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);
 
        assert_eq!(cpu.register_x, 1)
    }

   #[test]
   fn test_5_ops_working_together() {
       let mut cpu = CPU::new();
       cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

       assert_eq!(cpu.register_x, 0xc1)
   }

}