use std::borrow::Cow;
use num_bigint::BigUint;
use unsigned_varint::decode as varint;
use vexa_shared::bytereader::ByteReader;

pub struct RAM {
    memory: Vec<u8>,
    pub size: usize,
}

impl ByteReader for RAM {
    fn read_u8(&mut self, addr: usize) -> (u8, usize) {
        (self.peek(addr, 1)[0], 1)
    }

    fn read_u64(&mut self, addr: usize) -> (u64, usize) {
        (u64::from_be_bytes((&*self.peek(addr, 8)).try_into().unwrap()), 8)
    }

    fn read_biguint(&mut self, addr: usize) -> (BigUint, usize) {
        let (len, off1) = self._read_varint_usize(addr);
        let v = BigUint::from_bytes_be(&*self.peek(addr + off1, len));
        (v, off1 + len)
    }
}

impl RAM {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            memory: vec![0; size],
        }
    }

    pub fn total_size(&self) -> usize {
        self.memory.len()
    }

    pub fn put(&mut self, addr: usize, data: &[u8]) {
        let end = addr + data.len();
        if end <= self.memory.len() {
            self.memory[addr..end].copy_from_slice(data);  // memcpy, extremely fast
        } else {
            // wrapping case, rare
            let len = self.memory.len();
            for (i, byte) in data.iter().enumerate() {
                self.memory[(addr + i) % len] = *byte;
            }
        }
    }

    pub fn peek(&self, addr: usize, amount: usize) -> Cow<[u8]> {
        if addr + amount <= self.memory.len() {
            Cow::Borrowed(&self.memory[addr..addr + amount])
        } else {
            // slow wrapping path
            let len = self.memory.len();
            let mut result = Vec::with_capacity(amount);
            for i in 0..amount {
                result.push(self.memory[(addr + i) % len]);
            }
            Cow::Owned(result)
        }
    }

    fn _read_varint_usize(&mut self, addr: usize) -> (usize, usize) {
        let mut buf = [0u8; 10];
        let mut maddr = addr;
        let mut i = 0;
        loop {
            let (byte, off) = self.read_u8(maddr);
            buf[i] = byte;
            i += 1;
            maddr += off;
            if byte & 0x80 == 0 {break;}
        }
        (varint::usize(&buf)
            .map(|(val, _)| val)
            .unwrap(), maddr - addr)
    }
}
