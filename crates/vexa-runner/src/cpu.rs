use num_bigint::BigUint;
use num_traits::{ToBytes, ToPrimitive};
use vexa_shared::bytereader::ByteReader;
use crate::ram::RAM;
use vexa_shared::opcodes::Opcode;
use vexa_shared::operand_types::Operand;
use vexa_shared::registers::{LongRegisters, Registers};

pub struct CPU {
    // TODO: also get rid of this in the nearest future and move to the bus/motherboard thing I mentioned
    pub memory: RAM,
    // Rethink this later. Creating and destroying CPU for each different thing sounds like a bad idea
    // Tho, fun way of solving multithreading, just add some sort of "write-code-to-execute" function or something
    pub code: RAM,
    pub update_texture: bool,
    pub registers: RegisterMemory,
}

const RAM_SIZE: usize = 10 * 1024;  // 10KB
const STACK_SIZE: usize = 2 * 1024;  // 2KB

impl CPU {
    pub fn new(src: Vec<u8>) -> Self {
        let memory = RAM::new(RAM_SIZE + STACK_SIZE);
        let mut code = RAM::new(src.len());
        code.put(0, src.as_slice());
        let mut regs = RegisterMemory::new();
        regs.write_reg_long(LongRegisters::SP, RAM_SIZE as u64 + 1);

        Self {
            memory,
            update_texture: false,
            code,
            registers: regs
        }
    }

    fn read_u8(&mut self) -> u8 {
        let (v, a) = self.code.read_u8(self.registers.read_pc());
        self.registers.advance_pc(a);
        v
    }

    fn read_u64(&mut self) -> u64 {
        let (v, a) = self.code.read_u64(self.registers.read_pc());
        self.registers.advance_pc(a);
        v
    }
    
    fn parse_operand(&mut self) -> Operand {
        let (v, a) = Operand::from_bytes(&mut self.code, self.registers.read_pc());
        self.registers.advance_pc(a);
        v
    }

    pub fn push8(&mut self, value: u8) {
        let sp = self.registers.read_reg_long(LongRegisters::SP) as usize;
        self.memory.put(sp, &[value]);
        self.registers.write_reg_long(LongRegisters::SP, sp as u64 + 1);
    }

    pub fn pop8(&mut self) -> u8 {
        let sp = self.registers.read_reg_long(LongRegisters::SP) as usize;
        let data = self.memory.peek(sp - 1, 1)[0];
        self.registers.write_reg_long(LongRegisters::SP, sp as u64 - 1);
        data
    }

    pub fn push64(&mut self, value: u64) {
        let sp = self.registers.read_reg_long(LongRegisters::SP);
        self.memory.put(sp as usize, &value.to_be_bytes());
        self.registers.write_reg_long(LongRegisters::SP, sp + 8);
    }

    pub fn pop64(&mut self) -> u64 {
        let sp = self.registers.read_reg_long(LongRegisters::SP) - 8;
        self.registers.write_reg_long(LongRegisters::SP, sp);
        u64::from_be_bytes((&*self.memory.peek(sp as usize, 8)).try_into().unwrap())
    }

    pub fn step(&mut self) -> bool {
        let opcode = Opcode::from_bytecode(self.read_u8()).unwrap();
        match opcode {
            Opcode::Noop => {} // Noop
            Opcode::Hlt => {
                self.registers.move_pc(-1);  // read_u8 moves to forward, we bring it back to hlt
            }
            Opcode::Exit => {
                return false;
            }
            Opcode::Mov => {
                let src = self.parse_operand();
                let dest = self.parse_operand();

                match (src, dest) {
                    (Operand::Immediate(v), Operand::Address(addr)) => {
                        self.memory.put(addr as usize, &[v]);
                    }
                    (Operand::LongImmediate(v), Operand::Address(addr)) => {
                        self.memory.put(addr as usize, &v.to_be_bytes());
                    }
                    (Operand::LongerImmediate(v), Operand::Address(addr)) => {
                        self.memory.put(addr as usize, &v.to_be_bytes());
                    }
                    (Operand::Register(reg), Operand::Address(addr)) => {
                        self.memory.put(addr as usize, &[self.registers.read_reg(reg)])
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        self.memory.put(addr as usize, &self.registers.read_reg_long(reg).to_be_bytes())
                    }
                    (Operand::Immediate(v), Operand::IndirectAddress(reg)) => {
                        self.memory.put(self.registers.read_reg_long(reg) as usize, &[v]);
                    }
                    (Operand::LongImmediate(v), Operand::IndirectAddress(reg)) => {
                        self.memory.put(self.registers.read_reg_long(reg) as usize, &v.to_be_bytes());
                    }
                    (Operand::LongerImmediate(v), Operand::IndirectAddress(reg)) => {
                        self.memory.put(self.registers.read_reg_long(reg) as usize, &v.to_be_bytes());
                    }
                    (Operand::Register(reg1), Operand::IndirectAddress(reg2)) => {
                        self.memory.put(self.registers.read_reg_long(reg2) as usize, &[self.registers.read_reg(reg1)]);
                    }
                    (Operand::LongRegister(reg1), Operand::IndirectAddress(reg2)) => {
                        self.memory.put(self.registers.read_reg_long(reg2) as usize, &self.registers.read_reg_long(reg1).to_be_bytes())
                    }
                    (Operand::Address(addr), Operand::Register(reg)) => {
                        self.registers.write_reg(reg, self.memory.peek(addr as usize, 1)[0])
                    }
                    (Operand::Address(addr), Operand::LongRegister(reg)) => {
                        self.registers.write_reg_long(reg, u64::from_be_bytes((&self.memory.peek(addr as usize, 8)[..8]).try_into().unwrap()));
                    }
                    (Operand::IndirectAddress(reg1), Operand::Register(reg2)) => {
                        self.registers.write_reg(reg2, self.memory.peek(self.registers.read_reg_long(reg1) as usize, 1)[0])
                    }
                    (Operand::IndirectAddress(reg1), Operand::LongRegister(reg2)) => {
                        self.registers.write_reg_long(reg2, u64::from_be_bytes((&self.memory.peek(self.registers.read_reg_long(reg1) as usize, 8)[..8]).try_into().unwrap()));
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        self.registers.write_reg(reg2, self.registers.read_reg(reg1))
                    }
                    (Operand::LongImmediate(v), Operand::LongRegister(reg)) => {
                        self.registers.write_reg_long(reg, v);
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        self.registers.write_reg_long(reg2, self.registers.read_reg_long(reg1))
                    }
                    (Operand::Immediate(v), Operand::Register(reg)) => {
                        self.registers.write_reg(reg, v);
                    }
                    (_, _) => panic!()
                }
            }
            Opcode::Trunc => {
                let src = self.parse_operand();
                let dest = self.parse_operand();

                match (src, dest) {
                    (Operand::LongImmediate(v), Operand::Register(reg)) => {
                        self.registers.write_reg(reg, *v.to_be_bytes().last().unwrap());
                    }
                    (Operand::LongerImmediate(v), Operand::Register(reg)) => {
                        self.registers.write_reg(reg, *v.to_be_bytes().last().unwrap());
                    }
                    (Operand::LongRegister(reg1), Operand::Register(reg2)) => {
                        self.registers.write_reg(reg2, *self.registers.read_reg_long(reg1).to_be_bytes().last().unwrap())
                    }
                    (Operand::LongerImmediate(v), Operand::LongRegister(reg)) => {
                        let input = v.to_be_bytes();
                        let (int_bytes, _) = input.split_at(size_of::<u64>());
                        self.registers.write_reg_long(reg, u64::from_be_bytes(int_bytes.try_into().unwrap()))
                    }
                    (_, _) => panic!()
                }
            }
            Opcode::Ext => {
                let src = self.parse_operand();
                let dest = self.parse_operand();

                match (src, dest) {
                    (Operand::Address(addr), Operand::LongRegister(reg)) => {
                        self.registers.write_reg_long(reg, self.memory.peek(addr as usize, 1)[0] as u64)
                    }
                    (Operand::IndirectAddress(reg1), Operand::LongRegister(reg2)) => {
                        self.registers.write_reg_long(reg2, self.memory.peek(self.registers.read_reg_long(reg1) as usize, 1)[0] as u64)
                    }
                    (Operand::Immediate(val), Operand::LongRegister(reg)) => {
                        self.registers.write_reg_long(reg, val as u64)
                    }
                    (Operand::Register(reg1), Operand::LongRegister(reg2)) => {
                        self.registers.write_reg_long(reg2, self.registers.read_reg(reg1) as u64)
                    }
                    (_, _) => panic!()
                }
            }
            Opcode::Copy => {
                let length = match self.parse_operand() {
                    Operand::Immediate(v) => {BigUint::from(v)}
                    Operand::LongImmediate(v) => {BigUint::from(v)}
                    Operand::LongerImmediate(v) => {v}
                    Operand::Register(reg) => {BigUint::from(self.registers.read_reg(reg))}
                    Operand::LongRegister(reg) => {BigUint::from(self.registers.read_reg_long(reg))}
                    _ => {panic!()}
                };
                let addr1 = match self.parse_operand() {
                    Operand::Address(addr) => addr,
                    Operand::IndirectAddress(reg) => self.registers.read_reg_long(reg),
                    _ => panic!()
                };
                let addr2 = match self.parse_operand() {
                    Operand::Address(addr) => addr,
                    Operand::IndirectAddress(reg) => self.registers.read_reg_long(reg),
                    _ => panic!()
                };

                let mut offset = BigUint::ZERO;
                let mut remaining = length.clone();

                while remaining > BigUint::ZERO {
                    let chunk = remaining.clone().min(BigUint::from(usize::MAX)).to_usize().unwrap();
                    let data = self.memory.peek((addr1 + offset.to_u64().unwrap()) as usize, chunk).to_vec();
                    self.memory.put(
                        (addr2 + offset.to_u64().unwrap()) as usize,
                        &*data
                    );
                    offset += chunk;
                    remaining -= chunk;
                }
            }
            Opcode::Add => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let (result, overflow) = val1.overflowing_add(val2);
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let (result, overflow) = val1.overflowing_add(val2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let (result, overflow) = self.registers.read_reg(reg1).overflowing_add(self.registers.read_reg(reg2));
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg1).overflowing_add(self.registers.read_reg_long(reg2));
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) | (Operand::Register(reg), Operand::Immediate(val)) => {
                        let (result, overflow) = self.registers.read_reg(reg).overflowing_add(val);
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) | (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg).overflowing_add(val);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg).overflowing_add(addr);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (_, _) => {panic!()}
                }
            }
            Opcode::Sub => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let (result, overflow) = val1.overflowing_sub(val2);
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let (result, overflow) = val1.overflowing_sub(val2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let (result, overflow) = self.registers.read_reg(reg1).overflowing_sub(self.registers.read_reg(reg2));
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg1).overflowing_sub(self.registers.read_reg_long(reg2));
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) => {
                        let (result, overflow) = val.overflowing_sub(self.registers.read_reg(reg));
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::Register(reg), Operand::Immediate(val)) => {
                        let (result, overflow) = self.registers.read_reg(reg).overflowing_sub(val);
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg).overflowing_sub(val);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let (result, overflow) = val.overflowing_sub(self.registers.read_reg_long(reg));
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg).overflowing_sub(addr);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (_, _) => {panic!()}
                }
            }
            Opcode::Mul => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let (result, overflow) = val1.overflowing_mul(val2);
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let (result, overflow) = val1.overflowing_mul(val2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let (result, overflow) = self.registers.read_reg(reg1).overflowing_mul(self.registers.read_reg(reg2));
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg1).overflowing_mul(self.registers.read_reg_long(reg2));
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) | (Operand::Register(reg), Operand::Immediate(val)) => {
                        let (result, overflow) = self.registers.read_reg(reg).overflowing_mul(val);
                        self.registers.write_reg(Registers::A, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) | (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg).overflowing_mul(val);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        let (result, overflow) = self.registers.read_reg_long(reg).overflowing_mul(addr);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                        self.registers.write_reg(Registers::Z, if overflow { 1 } else { 0 });
                    }
                    (_, _) => {panic!()}
                }
            }
            Opcode::Div => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let (div, modulo) = (val1 / val2, val1 % val2);
                        self.registers.write_reg(Registers::A, div);
                        self.registers.write_reg(Registers::X, modulo);
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let (div, modulo) = (val1 / val2, val1 % val2);
                        self.registers.write_reg_long(LongRegisters::LL1, div);
                        self.registers.write_reg_long(LongRegisters::LL2, modulo);
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let (val1, val2) = (self.registers.read_reg(reg1), self.registers.read_reg(reg2));
                        let (div, modulo) = (val1 / val2, val1 % val2);
                        self.registers.write_reg(Registers::A, div);
                        self.registers.write_reg(Registers::X, modulo);
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let (val1, val2) = (self.registers.read_reg_long(reg1), self.registers.read_reg_long(reg2));
                        let (div, modulo) = (val1 / val2, val1 % val2);
                        self.registers.write_reg_long(LongRegisters::LL1, div);
                        self.registers.write_reg_long(LongRegisters::LL2, modulo);
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) | (Operand::Register(reg), Operand::Immediate(val)) => {
                        let (val1, val2) = (self.registers.read_reg(reg), val);
                        let (div, modulo) = (val1 / val2, val1 % val2);
                        self.registers.write_reg(Registers::A, div);
                        self.registers.write_reg(Registers::X, modulo);
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) | (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let (val1, val2) = (self.registers.read_reg_long(reg), val);
                        let (div, modulo) = (val1 / val2, val1 % val2);
                        self.registers.write_reg_long(LongRegisters::LL1, div);
                        self.registers.write_reg_long(LongRegisters::LL2, modulo);
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        let (val1, val2) = (self.registers.read_reg_long(reg), addr);
                        let (div, modulo) = (val1 / val2, val1 % val2);
                        self.registers.write_reg_long(LongRegisters::LL1, div);
                        self.registers.write_reg_long(LongRegisters::LL2, modulo);
                    }
                    (_, _) => {panic!()}
                }
            }
            Opcode::And => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let result = val1 & val2;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let result = val1 & val2;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let result = self.registers.read_reg(reg1) & self.registers.read_reg(reg2);
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let result = self.registers.read_reg_long(reg1) & self.registers.read_reg_long(reg2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) | (Operand::Register(reg), Operand::Immediate(val)) => {
                        let result = self.registers.read_reg(reg) & val;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) | (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let result = self.registers.read_reg_long(reg) & val;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        let result = self.registers.read_reg_long(reg) & addr;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (_, _) => {panic!()}
                }
            }
            Opcode::Or => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let result = val1 | val2;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let result = val1 | val2;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let result = self.registers.read_reg(reg1) | self.registers.read_reg(reg2);
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let result = self.registers.read_reg_long(reg1) | self.registers.read_reg_long(reg2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) | (Operand::Register(reg), Operand::Immediate(val)) => {
                        let result = self.registers.read_reg(reg) | val;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) | (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let result = self.registers.read_reg_long(reg) | val;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        let result = self.registers.read_reg_long(reg) | addr;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (_, _) => {panic!()}
                }
            }
            Opcode::Xor => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let result = val1 ^ val2;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let result = val1 ^ val2;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let result = self.registers.read_reg(reg1) ^ self.registers.read_reg(reg2);
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let result = self.registers.read_reg_long(reg1) ^ self.registers.read_reg_long(reg2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) | (Operand::Register(reg), Operand::Immediate(val)) => {
                        let result = self.registers.read_reg(reg) ^ val;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) | (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let result = self.registers.read_reg_long(reg) ^ val;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg), Operand::Address(addr)) => {
                        let result = self.registers.read_reg_long(reg) ^ addr;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (_, _) => {panic!()}
                }
            }
            Opcode::Not => {
                match self.parse_operand() {
                    Operand::Immediate(val) => {
                        self.registers.write_reg(Registers::A, !val);
                    }
                    Operand::LongImmediate(val) => {
                        self.registers.write_reg_long(LongRegisters::LL1, !val);
                    }
                    Operand::Register(reg) => {
                        self.registers.write_reg(Registers::A, !self.registers.read_reg(reg));
                    }
                    Operand::LongRegister(reg) => {
                        self.registers.write_reg_long(LongRegisters::LL1, !self.registers.read_reg_long(reg))
                    }
                    _ => panic!()
                }
            }
            Opcode::Shr => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let result = val1 >> val2;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let result = self.registers.read_reg(reg1) >> self.registers.read_reg(reg2);
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::Register(reg), Operand::Immediate(val)) => {
                        let result = self.registers.read_reg(reg) >> val;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) => {
                        let result = val >> self.registers.read_reg(reg);
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongImmediate(val1), Operand::Immediate(val2)) => {
                        let result = val1 >> val2;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongImmediate(val), Operand::Register(reg)) => {
                        let result = val >> self.registers.read_reg(reg);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg), Operand::Immediate(val)) => {
                        let result = self.registers.read_reg_long(reg) >> val;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg1), Operand::Register(reg2)) => {
                        let result = self.registers.read_reg_long(reg1) >> self.registers.read_reg(reg2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let result = self.registers.read_reg_long(reg1) >> self.registers.read_reg_long(reg2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) => {
                        let result = self.registers.read_reg_long(reg) >> val;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let result = val >> self.registers.read_reg_long(reg);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let result = val1 >> val2;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (_, _) => panic!()
                }
            }
            Opcode::Shl => {
                let op1 = self.parse_operand();
                let op2 = self.parse_operand();

                match (op1, op2) {
                    (Operand::Immediate(val1), Operand::Immediate(val2)) => {
                        let result = val1 << val2;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::Register(reg1), Operand::Register(reg2)) => {
                        let result = self.registers.read_reg(reg1) >> self.registers.read_reg(reg2);
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::Register(reg), Operand::Immediate(val)) => {
                        let result = self.registers.read_reg(reg) << val;
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::Immediate(val), Operand::Register(reg)) => {
                        let result = val << self.registers.read_reg(reg);
                        self.registers.write_reg(Registers::A, result);
                    }
                    (Operand::LongImmediate(val1), Operand::Immediate(val2)) => {
                        let result = val1 << val2;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongImmediate(val), Operand::Register(reg)) => {
                        let result = val << self.registers.read_reg(reg);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg), Operand::Immediate(val)) => {
                        let result = self.registers.read_reg_long(reg) << val;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg1), Operand::Register(reg2)) => {
                        let result = self.registers.read_reg_long(reg1) << self.registers.read_reg(reg2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg1), Operand::LongRegister(reg2)) => {
                        let result = self.registers.read_reg_long(reg1) << self.registers.read_reg_long(reg2);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongRegister(reg), Operand::LongImmediate(val)) => {
                        let result = self.registers.read_reg_long(reg) << val;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongImmediate(val), Operand::LongRegister(reg)) => {
                        let result = val << self.registers.read_reg_long(reg);
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (Operand::LongImmediate(val1), Operand::LongImmediate(val2)) => {
                        let result = val1 << val2;
                        self.registers.write_reg_long(LongRegisters::LL1, result);
                    }
                    (_, _) => panic!()
                }
            }
            Opcode::Jmp => {
                match self.parse_operand() {
                    Operand::Address(addr) => {self.registers.set_pc(addr as usize)},
                    Operand::IndirectAddress(reg) => {self.registers.set_pc(self.registers.read_reg_long(reg) as usize)},
                    _ => panic!()
                }
            },
            Opcode::Jnz => {
                // Man I love how this is implemented. It's the most cursed while still readable piece of code I've ever seen
                if match self.parse_operand() {
                    Operand::Register(reg) => {self.registers.read_reg(reg) != 0}
                    Operand::LongRegister(reg) => {self.registers.read_reg_long(reg) != 0}
                    _ => panic!()
                } {
                    match self.parse_operand() {  // And yes, I'm gonna keep this repetition here so it looks more cursed :P
                        Operand::Address(addr) => {self.registers.set_pc(addr as usize)},
                        Operand::IndirectAddress(reg) => {self.registers.set_pc(self.registers.read_reg_long(reg) as usize)},
                        _ => panic!()
                    }
                } else {
                    let _ = self.parse_operand(); // Just so I can keep cool cursed `if match` 😎
                }
            }
            Opcode::Jz => {
                if match self.parse_operand() {
                    Operand::Register(reg) => {self.registers.read_reg(reg) == 0}
                    Operand::LongRegister(reg) => {self.registers.read_reg_long(reg) == 0}
                    _ => panic!()
                } {
                    match self.parse_operand() {
                        Operand::Address(addr) => {self.registers.set_pc(addr as usize)},
                        Operand::IndirectAddress(reg) => {self.registers.set_pc(self.registers.read_reg_long(reg) as usize)},
                        _ => panic!()
                    }
                } else {
                    let _ = self.parse_operand();
                }
            }
            Opcode::PUSH => {
                let value = match self.parse_operand() {
                    Operand::Immediate(val) => val,
                    Operand::Register(reg) => {self.registers.read_reg(reg)}
                    _ => panic!()
                };

                self.push8(value);
            }
            Opcode::POP => {
                let value = self.pop8();
                self.registers.write_reg(Registers::A, value);
            }
            Opcode::PUSH64 => {
                let value = match self.parse_operand() {
                    Operand::LongImmediate(val) => val,
                    Operand::LongRegister(reg) => {self.registers.read_reg_long(reg)}
                    _ => panic!()
                };

                self.push64(value);
            }
            Opcode::POP64 => {
                let value = self.pop64();
                self.registers.write_reg_long(LongRegisters::LL1, value);
            }
            Opcode::CALL => {
                let target = match self.parse_operand() {
                    Operand::Address(addr) => addr,
                    Operand::IndirectAddress(reg) => self.registers.read_reg_long(reg),
                    _ => panic!()
                };
                let pc = self.registers.read_pc();
                self.push64(pc as u64);
                self.registers.set_pc(target as usize);
            }
            Opcode::RET => {
                let addr = self.pop64();
                self.registers.set_pc(addr as usize);
            }
            Opcode::LOG => {
                match self.parse_operand() {
                    Operand::Address(addr) => {
                        println!("[DEBUG] Address: ${}; 8-bit value: {}; 64-bit value: {}",
                                 addr,
                                 self.memory.peek(addr as usize, 1)[0],
                                 u64::from_be_bytes((&self.memory.peek(addr as usize, 8)[..8]).try_into().unwrap()))
                    }
                    Operand::Immediate(imm) => {
                        println!("[DEBUG] {}", imm);
                    }
                    Operand::LongImmediate(imm) => {
                        println!("[DEBUG] {}", imm);
                    }
                    Operand::LongerImmediate(imm) => {
                        println!("[DEBUG] {}", imm);
                    }
                    Operand::Register(reg) => {
                        println!("[DEBUG] REG({:?}) = {}", reg, self.registers.read_reg(reg));
                    }
                    Operand::LongRegister(reg) => {
                        println!("[DEBUG] LONG_REG({:?}) = {}", reg, self.registers.read_reg_long(reg));
                    }
                    Operand::IndirectAddress(reg) => {
                        let addr = self.registers.read_reg_long(reg) as usize;
                        println!("[DEBUG] LONG_REG({:?}) = Address: ${}; 8-bit value: {}; 64-bit value: {}",
                                 reg,
                                 addr,
                                 self.memory.peek(addr as usize, 1)[0],
                                 u64::from_be_bytes((&self.memory.peek(addr as usize, 8)[..8]).try_into().unwrap()))
                    }
                }
            }
            Opcode::LINE => {
                let Operand::Immediate(value) = self.parse_operand() else { panic!("Corrupted program ig...") };
                println!("{}", "=".repeat(value as usize));
            }
        }
        true
    }
}

pub struct RegisterMemory {
    pub registers: [u8; 0x8 + 1],
    pub long_registers: [u64; 0x6 + 1],
}

impl RegisterMemory {
    pub fn new() -> RegisterMemory {
        RegisterMemory {
            registers: [0; 0x8 + 1],
            long_registers: [0; 0x6 + 1],
        }
    }

    fn read_reg(&self, reg: Registers) -> u8 {
        self.registers[reg.to_bytecode() as usize]
    }
    fn read_reg_long(&self, reg: LongRegisters) -> u64 {
        self.long_registers[reg.to_bytecode() as usize]
    }

    fn write_reg(&mut self, reg: Registers, value: u8) {
        self.registers[reg.to_bytecode() as usize] = value;
    }
    
    fn write_reg_long(&mut self, reg: LongRegisters, value: u64) {
        self.long_registers[reg.to_bytecode() as usize] = value;
    }
    
    fn read_pc(&self) -> usize {
        self.read_reg_long(LongRegisters::PC) as usize
    }
    
    fn set_pc(&mut self, value: usize) {
        self.write_reg_long(LongRegisters::PC, value as u64);
    }
    
    fn advance_pc(&mut self, change: usize) {
        self.write_reg_long(LongRegisters::PC, self.read_reg_long(LongRegisters::PC) + change as u64);
    }
    
    fn move_pc(&mut self, change: i32) {
        self.write_reg_long(LongRegisters::PC, (self.read_reg_long(LongRegisters::PC) as i32 + change) as u64);
    }
}

#[allow(unused_mut)]
pub fn entry_emulator(mut emulator: CPU) {
    loop {
        if !emulator.step() {
            break;
        }
    }
}