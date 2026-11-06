use num_bigint::BigUint;
use num_traits::ToBytes;
use unsigned_varint::encode as varint;
use crate::bytereader::ByteReader;
use crate::registers::{LongRegisters, Registers};

#[derive(PartialEq, Debug)]
pub enum OperandKind {
    Address,
    Immediate,  // 1 byte
    LongImmediate,  // 8 bytes
    LongerImmediate, // 9+ bytes
    Register,  // 1 byte
    LongRegister, // 8 bytes
    IndirectAddress // Address taken from LongRegister
}

#[derive(Debug)]
pub enum Operand {
    Address(u64),
    Immediate(u8),
    LongImmediate(u64),
    LongerImmediate(BigUint),
    Register(Registers),
    LongRegister(LongRegisters),
    IndirectAddress(LongRegisters),
}

impl Operand {
    pub fn kind(&self) -> OperandKind {
        match self {
            Operand::Address(_) => OperandKind::Address,
            Operand::Immediate(_) => OperandKind::Immediate,
            Operand::LongImmediate(_) => OperandKind::LongImmediate,
            Operand::LongerImmediate(_) => OperandKind::LongerImmediate,
            Operand::Register(_) => OperandKind::Register,
            Operand::LongRegister(_) => OperandKind::LongRegister,
            Operand::IndirectAddress(_) => OperandKind::IndirectAddress,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Operand::Immediate(imm) => vec![0x01, *imm],
            Operand::LongImmediate(imm) => {
                let mut out = vec![0x02];
                out.extend(imm.to_be_bytes());
                out
            }
            Operand::LongerImmediate(imm) => {
                let mut out = vec![0x03];
                let blob = imm.to_be_bytes();

                let mut buf = varint::usize_buffer();
                let encoded = varint::usize(blob.len(), &mut buf);

                out.extend(encoded);
                out.extend(blob);

                out
            }
            Operand::Address(addr) => {
                let mut out = vec![0xAD];
                out.extend(addr.to_be_bytes());
                out
            }
            Operand::Register(reg) => {vec![0x10, reg.to_bytecode()]}
            Operand::LongRegister(reg) => {vec![0x11, reg.to_bytecode()]}
            Operand::IndirectAddress(reg) => vec![0x12, reg.to_bytecode()]
        }
    }

    pub fn from_bytes(memory: &mut impl ByteReader, addr: usize) -> (Operand, usize) {
        let (v, offset) = memory.read_u8(addr);
        match v {
            0x01 => {
                let (v, a) = memory.read_u8(addr + offset);
                (Operand::Immediate(v), a + offset)
            },
            0x02 => {
                let (v, a) = memory.read_u64(addr + offset);
                (Operand::LongImmediate(v), a + offset)
            },
            0x03 => {
                let (v, a) = memory.read_biguint(addr + offset);
                (Operand::LongerImmediate(v), a + offset)
            },
            0xAD => {
                let (v, a) = memory.read_u64(addr + offset);
                (Operand::Address(v), a + offset)
            },
            0x10 => {
                let (v, a) = memory.read_u8(addr + offset);
                (Operand::Register(Registers::from_bytecode(v)), a + offset)
            },
            0x11 => {
                let (v, a) = memory.read_u8(addr + offset);
                (Operand::LongRegister(LongRegisters::from_bytecode(v)), a + offset)
            },
            0x12 => {
                let (v, a) = memory.read_u8(addr + offset);
                (Operand::IndirectAddress(LongRegisters::from_bytecode(v)), a + offset)
            },
            _ => panic!("Uh... Corrupted program ig")
        }
    }
}
