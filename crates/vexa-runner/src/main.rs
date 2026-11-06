#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod cpu;
mod ram;

use std::env;
use std::fs::read;
use crate::cpu::{entry_emulator, CPU};


fn main() {
    let mut args = env::args().collect::<Vec<String>>();
    args.remove(0);
    let emulator = CPU::new(read(args.get(0).unwrap_or(&"".to_string()).to_string()).unwrap());
    entry_emulator(emulator);
}
