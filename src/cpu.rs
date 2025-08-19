#![allow(dead_code)]
use crate::opcodes;

// Flag Constants
const FLAG_CARRY:            u8 = 0b0000_0001; // bit 0
const FLAG_ZERO:             u8 = 0b0000_0010; // bit 1
const FLAG_INTERRUPT_DISABLE:u8 = 0b0000_0100; // bit 2
const FLAG_DECIMAL_MODE:     u8 = 0b0000_1000; // bit 3
const FLAG_BREAK:            u8 = 0b0001_0000; // bit 4
const FLAG_UNUSED:           u8 = 0b0010_0000; // bit 5 (should always read as 1 on NES)
const FLAG_OVERFLOW:         u8 = 0b0100_0000; // bit 6
const FLAG_NEGATIVE:         u8 = 0b1000_0000; // bit 7

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

// For CPU, Bus, and anything that needs to act as memory
trait Mem {
    fn mem_read_u16(&mut self, address: u16) -> u16{
        let lo = self.mem_read(address) as u16;
        let hi = self.mem_read(address + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, address: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;

        self.mem_write(address, lo);
        self.mem_write(address + 1, hi);
    }

    fn mem_read(&self, address: u16) -> u8;

    fn mem_write(&mut self, address: u16, data: u8);
}

impl Mem for CPU {
    fn mem_read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn mem_write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub memory: [u8; 0xFFFF],
}

// CPU Interface
impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    pub fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
           AddressingMode::Immediate => self.program_counter,

           AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
          
           AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
        
           AddressingMode::ZeroPage_X => {
               let pos = self.mem_read(self.program_counter);
               let addr = pos.wrapping_add(self.register_x) as u16;
               addr
           }
           AddressingMode::ZeroPage_Y => {
               let pos = self.mem_read(self.program_counter);
               let addr = pos.wrapping_add(self.register_y) as u16;
               addr
           }

           AddressingMode::Absolute_X => {
               let base = self.mem_read_u16(self.program_counter);
               let addr = base.wrapping_add(self.register_x as u16);
               addr
           }
           AddressingMode::Absolute_Y => {
               let base = self.mem_read_u16(self.program_counter);
               let addr = base.wrapping_add(self.register_y as u16);
               addr
           }

           AddressingMode::Indirect_X => {
               let base = self.mem_read(self.program_counter);

               let ptr: u8 = (base as u8).wrapping_add(self.register_x);
               let lo = self.mem_read(ptr as u16);
               let hi = self.mem_read(ptr.wrapping_add(1) as u16);
               (hi as u16) << 8 | (lo as u16)
           }
           AddressingMode::Indirect_Y => {
               let base = self.mem_read(self.program_counter);

               let lo = self.mem_read(base as u16);
               let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
               let deref_base = (hi as u16) << 8 | (lo as u16);
               let deref = deref_base.wrapping_add(self.register_y as u16);
               deref
           }
         
           AddressingMode::NoneAddressing => {
               panic!("mode {:?} is not supported", mode);
           }
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
       self.load(program);
       self.reset();
       self.run();
    }

    // For Tests
    fn load_and_reset(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
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

    pub fn run(&mut self) {

        let opcode_map = &*opcodes::OPCODES_MAP;

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;
            let opcode = opcode_map.get(&code).expect(&format!("Code {:x} not in map.", code)); 

            match code {
                /* LDA */
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&opcode.mode);
                }
                
                /* STA */
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }

                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => return,
                _ => todo!("{} ({:x}) with mode {:?} not implemented yet.", opcode.mnemonic, opcode.code, opcode.mode),
            }

            // Ensures PC moves proper amount forward
            // Will not trigger during jump type opcodes.
            if self.program_counter == program_counter_state {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }

    
}


// Opcodes
impl CPU {
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
      
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
   }
   
   fn sta (&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
   }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
   }

   fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
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
        cpu.load_and_run(vec![0xA9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.check(FLAG_ZERO));
        assert!(!cpu.check(FLAG_NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.check(FLAG_ZERO));
    }

    #[test] 
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1000_0001, 0x00]);
        assert!(cpu.check(FLAG_NEGATIVE));
        assert!(cpu.check(!FLAG_ZERO));
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xaa, 0x00]);
        cpu.register_a = 10;
        cpu.run();

        assert_eq!(cpu.register_a, 10);
    }

    #[test]
    fn test_0xaa_tax_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xaa, 0x00]);
        cpu.register_a = 0b1000_0001;
        cpu.run();

        assert!(cpu.check(FLAG_NEGATIVE));
        assert!(!cpu.check(FLAG_ZERO));
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0;
        cpu.load_and_reset(vec![0xaa, 0x00]);
        cpu.run();

        assert!(cpu.check(FLAG_ZERO));
        assert!(!cpu.check(FLAG_NEGATIVE));
    }

    #[test]
    fn test_0xe8_inx_increment_x() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xe8, 0x00]);
        cpu.register_x = 10;
        cpu.run();

        assert_eq!(cpu.register_x, 11);
    }

    #[test]
    fn test_0xe8_inx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xe8, 0x00]);
        cpu.register_x = 0b1111_1111;
        cpu.run();

        assert!(cpu.check(FLAG_ZERO));
        assert!(!cpu.check(FLAG_NEGATIVE))
    }

    #[test]
    fn test_0xe8_inx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xe8, 0x00]);
        cpu.register_x = 0b1000_0001;
        cpu.run();
        assert!(cpu.check(FLAG_NEGATIVE));
        assert!(!cpu.check(FLAG_ZERO));
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xe8, 0xe8, 0x00]);
        cpu.register_x = 0xff;
        cpu.run();
        
        assert_eq!(cpu.register_x, 1)
    }

   #[test]
   fn test_5_ops_working_together() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

       assert_eq!(cpu.register_x, 0xc1)
   }

   #[test]
   fn test_sta_stores_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x10, 0x85, 0x69, 0x00]);
        assert_eq!(cpu.mem_read(0x69), 0x10);
   }
   

}