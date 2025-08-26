#![allow(dead_code)]
use crate::opcodes;

// Flag Constants
const FLAG_CARRY: u8 = 0b0000_0001; // bit 0
const FLAG_ZERO: u8 = 0b0000_0010; // bit 1
const FLAG_INTERRUPT_DISABLE: u8 = 0b0000_0100; // bit 2
const FLAG_DECIMAL_MODE: u8 = 0b0000_1000; // bit 3
const FLAG_BREAK: u8 = 0b0001_0000; // bit 4
const FLAG_UNUSED: u8 = 0b0010_0000; // bit 5 (should always read as 1 on NES)
const FLAG_OVERFLOW: u8 = 0b0100_0000; // bit 6
const FLAG_NEGATIVE: u8 = 0b1000_0000; // bit 7

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
    fn mem_read_u16(&mut self, address: u16) -> u16 {
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

// CPU Interface (Helpers, mostly)
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

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

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
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
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

    fn update_zero_flag(&mut self, result: u8) {
        let condition = result == 0;
        self.set_flag_if(FLAG_ZERO, condition);
    }

    fn update_negative_flag(&mut self, result: u8) {
        let condition = result & FLAG_NEGATIVE != 0;
        self.set_flag_if(FLAG_NEGATIVE, condition);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    // Returns true if the given flag(s) are all set.
    // Combine flags with pipes to test multiple at once
    fn check_flag(&self, flag: u8) -> bool {
        return self.status & flag == flag;
    }

    fn set_flag(&mut self, flag: u8) {
        self.status = self.status | flag;
    }

    fn clear_flag(&mut self, flag: u8) {
        self.status = self.status & !flag;
    }

    fn set_flag_if(&mut self, flag: u8, condition: bool) {
        if condition {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn set_register_x(&mut self, value: u8) {
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn set_register_y(&mut self, value: u8) {
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn set_program_counter(&mut self, value: u16) {
        self.program_counter = value;
    }

    pub fn run(&mut self) {
        let opcode_map = &*opcodes::OPCODES_MAP;

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;
            let opcode = opcode_map
                .get(&code)
                .expect(&format!("Code {:x} not in map.", code));

            match code {
                /* AND */
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&opcode.mode);
                }

                /* ASL */
                0x0A => self.asl_accumulator(),

                0x06 | 0x16 | 0x0E | 0x1E => {
                    self.asl(&opcode.mode);
                }

                /* BCC */
                0x90 => {
                    self.branch(!self.check_flag(FLAG_CARRY));
                }

                /* BCS */
                0xB0 => {
                    self.branch(self.check_flag(FLAG_CARRY));
                }

                /* BEQ */
                0xF0 => {
                    self.branch(self.check_flag(FLAG_ZERO));
                }

                /* BMI */
                0x30 => {
                    self.branch(self.check_flag(FLAG_NEGATIVE));
                }

                /* BNE */
                0xD0 => {
                    self.branch(!self.check_flag(FLAG_ZERO));
                }

                /* BPL */
                0x10 => {
                    self.branch(!self.check_flag(FLAG_NEGATIVE));
                }

                /* BVC */
                0x50 => self.branch(!self.check_flag(FLAG_OVERFLOW)),

                /* BVS */
                0x70 => self.branch(self.check_flag(FLAG_OVERFLOW)),

                /* BIT */
                0x24 | 0x2C => self.bit(&opcode.mode),

                /* Clear Flags */
                0x18 => self.clear_flag(FLAG_CARRY),
                0xD8 => self.clear_flag(FLAG_DECIMAL_MODE),
                0x58 => self.clear_flag(FLAG_INTERRUPT_DISABLE),
                0xB8 => self.clear_flag(FLAG_OVERFLOW),

                /* Comparisons */
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.compare(&opcode.mode, self.register_a); // CMP
                }

                0xE0 | 0xE4 | 0xEC => {
                    self.compare(&opcode.mode, self.register_x); // CPX
                }

                0xC0 | 0xC4 | 0xCC => {
                    self.compare(&opcode.mode, self.register_y); // CPY
                }

                /* Decrements */
                0xC6 | 0xD6 | 0xCE | 0xDE => {
                    self.dec(&opcode.mode)
                }

                0xCA => self.dex(&opcode.mode),
                
                0x88 => self.dey(&opcode.mode),

                /* LDA */
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&opcode.mode);
                }

                /* LDX */
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(&opcode.mode);
                }

                /* LDY */
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(&opcode.mode);
                }

                /* STA */
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }

                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => return,
                _ => todo!(
                    "{} (0x{:x}) with mode {:?}",
                    opcode.mnemonic,
                    opcode.code,
                    opcode.mode
                ),
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
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_register_a(self.register_a & value);
    }

    fn asl_accumulator(&mut self) {
        let mut value = self.register_a;
        if value >> 7 == 1 {
            self.set_flag(FLAG_CARRY);
        } else {
            self.clear_flag(FLAG_CARRY);
        }
        value = value << 1;
        self.set_register_a(value);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);
        if value >> 7 == 1 {
            self.set_flag(FLAG_CARRY);
        } else {
            self.clear_flag(FLAG_CARRY);
        }

        value = value << 1;
        self.mem_write(addr, value);
        self.update_negative_flag(value);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            let jump = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            self.program_counter = jump_addr;
        }
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        let masked_value = data & self.register_a;
        let bit6 = data & FLAG_OVERFLOW;
        let bit7 = data & FLAG_NEGATIVE;

        self.set_flag_if(FLAG_OVERFLOW, bit6 != 0);
        self.set_flag_if(FLAG_NEGATIVE, bit7 != 0);
        self.set_flag_if(FLAG_ZERO, masked_value == 0);
    }

    fn compare(&mut self, mode: &AddressingMode, compare_val: u8) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        if compare_val >= data {
            self.set_flag(FLAG_CARRY);
        } else {
            self.clear_flag(FLAG_CARRY);
        }

        self.update_zero_and_negative_flags(compare_val.wrapping_sub(data));
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let result = self.mem_read(addr).wrapping_sub(1);

        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn dex(&mut self, mode: &AddressingMode) {
        let result = self.register_x.wrapping_sub(1);
        self.set_register_x(result);
    }

    fn dey(&mut self, mode: &AddressingMode) {
        let result = self.register_y.wrapping_sub(1);
        self.set_register_y(result);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_a(value);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_x(value);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_y(value);
    }

    fn sta(&mut self, mode: &AddressingMode) {
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

        assert!(!cpu.check_flag(FLAG_ZERO));
        assert!(!cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        assert!(cpu.check_flag(FLAG_ZERO));
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1000_0001, 0x00]);

        assert!(cpu.check_flag(FLAG_NEGATIVE));
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

        assert!(cpu.check_flag(FLAG_NEGATIVE));
        assert!(!cpu.check_flag(FLAG_ZERO));
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0;
        cpu.load_and_reset(vec![0xaa, 0x00]);
        cpu.run();

        assert!(cpu.check_flag(FLAG_ZERO));
        assert!(!cpu.check_flag(FLAG_NEGATIVE));
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

        assert!(cpu.check_flag(FLAG_ZERO));
        assert!(!cpu.check_flag(FLAG_NEGATIVE))
    }

    #[test]
    fn test_0xe8_inx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xe8, 0x00]);
        cpu.register_x = 0b1000_0001;
        cpu.run();
        assert!(cpu.check_flag(FLAG_NEGATIVE));
        assert!(!cpu.check_flag(FLAG_ZERO));
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

    #[test]
    fn test_and_with_flags() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1111_1111, 0x29, 0b1000_1011, 0x00]);

        assert_eq!(cpu.register_a, 0b1000_1011);
        assert!(cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn test_asl_doubles_value() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b0000_0001, 0x0A, 0x00]);

        assert_eq!(cpu.register_a, 0b0000_0010);
    }

    #[test]
    fn test_asl_with_flags() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1100_0000, 0x0A, 0x00]);

        assert_eq!(cpu.register_a, 0b1000_0000);
        assert!(cpu.check_flag(FLAG_CARRY));
        assert!(cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    #[rustfmt::skip]
    fn test_bcc_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0b1000_0000, 0x0A, 0x90, 1, 0x00, 0xA9, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x00);

        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0b0100_0000, 0x0A, 0x90, 1, 0x00, 0xA9, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x10);
    }

    #[test]
    #[rustfmt::skip]
    fn test_bcs_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0b1000_0000, 0x0A, 0xB0, 1, 0x00, 0xA9, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x10);

        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0b0100_0000, 0x0A, 0xB0, 1, 0x00, 0xA9, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0b1000_0000);
    }

    #[test]
    #[rustfmt::skip]
    fn test_beq_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x11, 0xF0, 1, 0x00, 0xA9, 0x22, 0x00]);
        assert_eq!(cpu.register_a, 0x11);
        
        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0xF0, 1, 0x00, 0xA9, 0x22, 0x00]);
        assert_eq!(cpu.register_a, 0x22);
    }

    #[test]
    #[rustfmt::skip]
    fn test_bmi_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x01, 0x30, 1, 0x00, 0xA9, 0x02, 0x00]);
        assert_eq!(cpu.register_a, 0x01);
        
        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0b1000_0001, 0x30, 1, 0x00, 0xA9, 0x02, 0x00]);
        assert_eq!(cpu.register_a, 0x02);
    }

    #[test]
    #[rustfmt::skip]
    fn test_bne_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0xD0, 1, 0x00, 0xA9, 0x22, 0x00]);
        assert_eq!(cpu.register_a, 0x00);
        
        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x11, 0xD0, 1, 0x00, 0xA9, 0x22, 0x00]);
        assert_eq!(cpu.register_a, 0x22);
    }

    #[test]
    #[rustfmt::skip]
    fn test_bpl_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0b1000_0001, 0x10, 1, 0x00, 0xA9, 0x02, 0x00]);
        assert_eq!(cpu.register_a, 0b1000_0001);

        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x01, 0x10, 1, 0x00, 0xA9, 0x02, 0x00]);
        assert_eq!(cpu.register_a, 0x02);
    }

    #[test]
    #[rustfmt::skip]
    fn test_bvc_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xA9, 0x11, 0x50, 1, 0x00, 0xA9, 0x22, 0x00]);
        cpu.run();
        assert_eq!(cpu.register_a, 0x22);

        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xA9, 0x11, 0x50, 1, 0x00, 0xA9, 0x22, 0x00]);
        cpu.set_flag(FLAG_OVERFLOW);
        cpu.run();
        assert_eq!(cpu.register_a, 0x11);
    }

    #[test]
    #[rustfmt::skip]
    fn test_bvs_works() {
        // Doesn't Jump
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xA9, 0x11, 0x70, 1, 0x00, 0xA9, 0x22, 0x00]);
        cpu.set_flag(FLAG_OVERFLOW);
        cpu.run();
        assert_eq!(cpu.register_a, 0x22);
        
        // Jumps
        let mut cpu = CPU::new();
        cpu.load_and_reset(vec![0xA9, 0x11, 0x70, 1, 0x00, 0xA9, 0x22, 0x00]);
        cpu.run();
        assert_eq!(cpu.register_a, 0x11);
    }

    #[test]
    fn test_flag_clears() {
        let mut cpu = CPU::new();
        let test_flags = FLAG_CARRY | FLAG_DECIMAL_MODE | FLAG_INTERRUPT_DISABLE | FLAG_OVERFLOW;
        cpu.load_and_reset(vec![0x18, 0xD8, 0x58, 0xB8, 0x00]);

        cpu.set_flag(test_flags); // Turn on all flags
        assert!(cpu.check_flag(test_flags));
        cpu.run(); // Should clear all flags

        assert!(!cpu.check_flag(test_flags));
    }

    #[test]
    fn test_cmp_works() {
        let mut cpu = CPU::new();
        let reg_a_val = 10;
        let carry_setter: u8 = 5;
        let zero_setter: u8 = 10;
        let neg_setter: u8 = 15;

        cpu.load_and_run(vec![0xA9, reg_a_val, 0xC9, carry_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_CARRY));

        cpu.load_and_run(vec![0xA9, reg_a_val, 0xC9, zero_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_ZERO));

        cpu.load_and_run(vec![0xA9, reg_a_val, 0xC9, neg_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn test_cpx_works() {
        let mut cpu = CPU::new();
        let reg_x_val = 10;
        let carry_setter: u8 = 5;
        let zero_setter: u8 = 10;
        let neg_setter: u8 = 15;

        cpu.load_and_run(vec![0xA2, reg_x_val, 0xE0, carry_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_CARRY));

        cpu.load_and_run(vec![0xA2, reg_x_val, 0xE0, zero_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_ZERO));

        cpu.load_and_run(vec![0xA2, reg_x_val, 0xE0, neg_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn test_cpy_works() {
        let mut cpu = CPU::new();
        let reg_y_val = 10;
        let carry_setter: u8 = 5;
        let zero_setter: u8 = 10;
        let neg_setter: u8 = 15;

        cpu.load_and_run(vec![0xA0, reg_y_val, 0xC0, carry_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_CARRY));

        cpu.load_and_run(vec![0xA0, reg_y_val, 0xC0, zero_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_ZERO));

        cpu.load_and_run(vec![0xA0, reg_y_val, 0xC0, neg_setter, 0x00]);
        assert!(cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn test_bit_sets_v_n_and_clears_z() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xCF, 0x85, 0x10, 0x24, 0x10, 0x00]);
        assert!(!cpu.check_flag(FLAG_ZERO));
        assert!(cpu.check_flag(FLAG_OVERFLOW));
        assert!(cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn test_bit_sets_z_and_v_n() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xC0, 0x85, 0x10, 0xA9, 0x3F, 0x24, 0x10, 0x00]);
        assert!(cpu.check_flag(FLAG_ZERO));
        assert!(cpu.check_flag(FLAG_OVERFLOW));
        assert!(cpu.check_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn ldx_works_with_flags() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA2, 0b1000_0000, 0x00]);
        assert!(cpu.check_flag(FLAG_NEGATIVE));
        assert!(!cpu.check_flag(FLAG_ZERO));
        assert_eq!(cpu.register_x, 0b1000_0000);

        cpu.load_and_run(vec![0xA2, 0x00, 0x00]);
        assert!(cpu.check_flag(FLAG_ZERO));
        assert!(!cpu.check_flag(FLAG_NEGATIVE));
        assert_eq!(cpu.register_x, 0);
    }

    #[test]
    fn ldy_works_with_flags() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA0, 0b1000_0000, 0x00]);
        assert!(cpu.check_flag(FLAG_NEGATIVE));
        assert!(!cpu.check_flag(FLAG_ZERO));
        assert_eq!(cpu.register_y, 0b1000_0000);

        cpu.load_and_run(vec![0xA0, 0x00, 0x00]);
        assert!(cpu.check_flag(FLAG_ZERO));
        assert!(!cpu.check_flag(FLAG_NEGATIVE));
        assert_eq!(cpu.register_y, 0);
    }

    #[test]
    fn dec_works_with_flags() {
        todo!("");
    }

    #[test]
    fn dex_works_with_flags() {
        todo!("");
    }

    #[test]
    fn dey_works_with_flags() {
        todo!("");
    }
    
}
