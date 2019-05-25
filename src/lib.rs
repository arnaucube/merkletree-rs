use std::collections::HashMap;

#[macro_use]
extern crate arrayref;

extern crate tiny_keccak;

extern crate rustc_hex;

mod utils;
mod node;

// const TYPENODEROOT: u8 = 4;
const EMPTYNODEVALUE: [u8;32] = [0;32];

pub struct TestValue {
   bytes: Vec<u8>,
   index_length: u32,
}
pub trait Value {
   fn bytes(&self) -> &Vec<u8>;
   fn index_length(&self) -> u32;
}
impl Value for TestValue {
   fn bytes(&self) -> &Vec<u8> {
      &self.bytes
   }
   fn index_length(&self) -> u32 {
      self.index_length
   }
}

