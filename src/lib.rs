use std::collections::HashMap;

#[macro_use]
extern crate arrayref;

extern crate tiny_keccak;

extern crate rustc_hex;

mod utils;
mod node;

const TYPENODEEMPTY: u8 = 0;
const TYPENODENORMAL: u8 = 1;
const TYPENODEFINAL: u8 = 2;
const TYPENODEVALUE: u8 = 3;
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

#[allow(dead_code)]
pub struct Db {
   storage: HashMap<[u8;32], Vec<u8>>,
}
impl Db {
   pub fn insert(&mut self, k: [u8; 32], t: u8, il: u32, b: &mut Vec<u8>) {
      let mut v: Vec<u8>;
      v = [t].to_vec();
      let il_bytes = il.to_le_bytes();
      v.append(&mut il_bytes.to_vec()); // il_bytes are [u8;4] (4 bytes)
      v.append(b);
      self.storage.insert(k, v);
   }
   pub fn get(&self, k: &[u8;32]) -> (u8, u32, Vec<u8>) {
      if k.to_vec() == EMPTYNODEVALUE.to_vec() {
         return (0, 0, EMPTYNODEVALUE.to_vec());
      }
      match self.storage.get(k) {
         Some(x) => {
            let t = x[0];
            let il_bytes: [u8; 4] = [x[1], x[2], x[3], x[4]];
            let il = u32::from_le_bytes(il_bytes);
            let b = &x[5..];
            return (t, il, b.to_vec());
         },
         None => return (TYPENODEEMPTY, 0, EMPTYNODEVALUE.to_vec()),
      }
   }
} 

pub fn new_db()-> Db {
   let db = Db {
      storage: HashMap::new(),
   };
   db
}

pub struct MerkleTree {
   #[allow(dead_code)]
   root: [u8; 32],
   #[allow(dead_code)]
   num_levels: u32,
   #[allow(dead_code)]
   sto: Db,
}


pub fn new(num_levels: u32) -> MerkleTree {
   let mt = MerkleTree {
      root: EMPTYNODEVALUE,
      num_levels: num_levels,
      sto: new_db(),
   };
   mt
}
