use num_bigint::BigUint;

pub trait ByteReader {
    fn read_u8(&mut self, addr: usize) -> (u8, usize);
    fn read_u64(&mut self, addr: usize) -> (u64, usize);
    fn read_biguint(&mut self, addr: usize) -> (BigUint, usize);
}