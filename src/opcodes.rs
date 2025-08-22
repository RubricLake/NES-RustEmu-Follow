use crate::cpu::AddressingMode;
use std::collections::HashMap;

pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub len: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    fn new(code: u8, mnemonic: &'static str, len: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            code: code,
            mnemonic: mnemonic,
            len: len,
            cycles: cycles,
            mode: mode,
        }
    }
}
/* Copy and Paste
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::ZeroPage),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::ZeroPage_X),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::ZeroPage_Y),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::Absolute),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::Absolute_X),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::Absolute_Y),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::Indirect_X),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::Indirect_Y),
OpCode::new(0x00, "MNE", 0, 0, AddressingMode::NoneAddressing),

// +1 if page crossed

*/

// Opcode Table
lazy_static! {
    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        OpCode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),

        // Logical AND
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3D, "AND", 3, 4, AddressingMode::Absolute_X), // + 1 if page crossed
        OpCode::new(0x39, "AND", 3, 4, AddressingMode::Absolute_Y), // + 1 if page crossed
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, "AND", 2, 5, AddressingMode::Indirect_Y), // + 1 if page crossed

        // Arithmetic Shift Left
        OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::Absolute_X),

        /*
        Branching
        len +1 if branch succeeds (+2 if to a new page)
        */
        OpCode::new(0x90, "BCC", 2, 2, AddressingMode::NoneAddressing), // Branch if Carry Clear
        OpCode::new(0xB0, "BCS", 2, 2, AddressingMode::NoneAddressing), // Branch if Carry Set
        OpCode::new(0xF0, "BEQ", 2, 2, AddressingMode::NoneAddressing), // Branch if Equal
        OpCode::new(0x30, "BMI", 2, 2, AddressingMode::NoneAddressing), // Branch if Minus
        OpCode::new(0xD0, "BNE", 2, 2, AddressingMode::NoneAddressing), // Branch if Not Equal
        OpCode::new(0x10, "BPL", 2, 2, AddressingMode::NoneAddressing), // Branch if Positive
        OpCode::new(0x50, "BVC", 2, 2, AddressingMode::NoneAddressing), // Branch if Overflow Clear
        OpCode::new(0x70, "BVS", 2, 2, AddressingMode::NoneAddressing), // If Overflow set

        /* Clear Flags */
        OpCode::new(0x18, "CLC", 1, 2, AddressingMode::NoneAddressing), // Clear Carry
        OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NoneAddressing), // Clear Decimal Mode
        OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NoneAddressing), // Clear Interrupt Disable
        OpCode::new(0xB8, "CLV", 1, 2, AddressingMode::NoneAddressing), // Clear Overflow

        /* Comparisons */
        OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xDD, "CMP", 3, 4, AddressingMode::Absolute_X), // +1 if page crossed
        OpCode::new(0xD9, "CMP", 3, 4, AddressingMode::Absolute_Y), // +1 if page crossed
        OpCode::new(0xC1, "CMP", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xD1, "CMP", 2, 5, AddressingMode::Indirect_Y), // +1 if page crossed

        OpCode::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute),

        OpCode::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute),

        // Load Accumulator
        OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X), // +1 if page crossed
        OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y), // +1 if page crossed
        OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB1, "LDA", 2, 5, AddressingMode::Indirect_Y), // +1 if page crossed

        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8d, "STA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA", 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),
    ];

    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut m = HashMap::new();
        for op in &*CPU_OPS_CODES {
            m.insert(op.code, op);
        }
        m
    };


}
