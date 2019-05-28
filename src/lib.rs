use std::collections::HashMap;

#[macro_use]
extern crate arrayref;

extern crate tiny_keccak;

extern crate rustc_hex;

extern crate hex;

mod utils;
mod node;

const TYPENODEEMPTY: u8 = 0;
const TYPENODENORMAL: u8 = 1;
const TYPENODEFINAL: u8 = 2;
const TYPENODEVALUE: u8 = 3;
const EMPTYNODEVALUE: [u8;32] = [0;32];

pub struct TestValue {
   bytes: Vec<u8>,
   index_length: u32,
}
pub trait Value {
   fn bytes(&self) -> &Vec<u8>;
   fn index_length(&self) -> u32;
   fn hi(&self) -> [u8;32];
   fn ht(&self) -> [u8;32];
}
impl Value for TestValue {
   fn bytes(&self) -> &Vec<u8> {
      &self.bytes
   }
   fn index_length(&self) -> u32 {
      self.index_length
   }
   fn hi(&self) -> [u8;32] {
      utils::hash_vec(self.bytes().to_vec().split_at(self.index_length() as usize).0.to_vec())
   }
   fn ht(&self) -> [u8;32] {
      utils::hash_vec(self.bytes().to_vec())
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
            (t, il, b.to_vec())
         },
         None => (TYPENODEEMPTY, 0, EMPTYNODEVALUE.to_vec()),
      }
   }
} 

pub fn new_db()-> Db {
   Db {
      storage: HashMap::new(),
   }
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
   MerkleTree {
      root: EMPTYNODEVALUE,
      num_levels,
      sto: new_db(),
   }
}

impl MerkleTree {
   pub fn add(&mut self, v: &TestValue) {
      #![allow(unused_variables)]
      #[allow(dead_code)]

      // println!("adding value: {:?}", v.bytes());
      // add the leaf that we are adding
      self.sto.insert(v.ht(), TYPENODEVALUE, v.index_length(), &mut v.bytes().to_vec());

      let index = v.index_length() as usize;
      let hi = v.hi();
      let ht = v.ht();
      let path = utils::get_path(self.num_levels, hi);
      let mut siblings: Vec<[u8;32]> = Vec::new();


      let mut node_hash = self.root;

      for i in (0..=self.num_levels-2).rev() {
         // get node
         let (t, il, node_bytes) = self.sto.get(&node_hash);
         if t == TYPENODEFINAL {
            let hi_child = utils::hash_vec(node_bytes.to_vec().split_at(il as usize).0.to_vec());
            let path_child = utils::get_path(self.num_levels, hi_child);
            let pos_diff = utils::compare_paths(&path_child, &path);
            if pos_diff == 999 { // TODO use a match here, and instead of 999 return something better
               println!("compare paths err");
               return;
            }
            let final_node_1_hash = utils::calc_hash_from_leaf_and_level(pos_diff, &path_child, utils::hash_vec(node_bytes.to_vec()));
            self.sto.insert(final_node_1_hash, TYPENODEFINAL, il, &mut node_bytes.to_vec());
            let final_node_2_hash = utils::calc_hash_from_leaf_and_level(pos_diff, &path, v.ht());
            self.sto.insert(final_node_2_hash, TYPENODEFINAL, v.index_length(), &mut v.bytes().to_vec());

            // parent node
            let parent_node: node::TreeNode;
            if path[pos_diff as usize] {
               parent_node = node::TreeNode {
                  child_l: final_node_1_hash,
                  child_r: final_node_2_hash,
               }} else {
               parent_node = node::TreeNode {
                  child_l: final_node_2_hash,
                  child_r: final_node_1_hash,
               }}
            let empties = utils::get_empties_between_i_and_pos(i, pos_diff+1);
            for empty in &empties {
               siblings.push(*empty);
            }

            let path_from_pos_diff = utils::cut_path(&path, (pos_diff +1) as usize);
            self.root = self.replace_leaf(path_from_pos_diff, siblings.clone(), parent_node.ht(), TYPENODENORMAL, 0, &mut parent_node.bytes().to_vec());
            return;
         }

         let node = node::parse_node_bytes(node_bytes);

         let sibling: [u8;32];
         if path[i as usize] {
            node_hash = node.child_l;
            sibling = node.child_r;
         } else {
            sibling = node.child_l;
            node_hash = node.child_r;
         }
         siblings.push(*array_ref!(sibling, 0, 32));
         if node_hash == EMPTYNODEVALUE {
            if i==self.num_levels-2 && siblings[siblings.len()-1]==EMPTYNODEVALUE {
               let final_node_hash = utils::calc_hash_from_leaf_and_level(i+1, &path, v.ht());
               self.sto.insert(final_node_hash, TYPENODEFINAL, v.index_length(), &mut v.bytes().to_vec());
               self.root = final_node_hash;
               return;
            }
            let final_node_hash = utils::calc_hash_from_leaf_and_level(i, &path, v.ht());
            let path_from_i = utils::cut_path(&path, i as usize);
            self.root = self.replace_leaf(path_from_i, siblings.clone(), final_node_hash, TYPENODEFINAL, v.index_length(), &mut v.bytes().to_vec());
            return;
         }
      }
      self.root = self.replace_leaf(path, siblings, v.ht(), TYPENODEVALUE, v.index_length(), &mut v.bytes().to_vec());
   }

   #[allow(dead_code)]
   pub fn replace_leaf(&mut self, path: Vec<bool>, siblings: Vec<[u8;32]>, leaf_hash: [u8;32], node_type: u8, index_length: u32, leaf_value: &mut Vec<u8>) -> [u8;32] {
      self.sto.insert(leaf_hash, node_type, index_length, leaf_value);
      let mut curr_node = leaf_hash;

      for i in 0..siblings.len() {
         if path[i as usize] {
            let node = node::TreeNode {
               child_l: curr_node,
               child_r: siblings[siblings.len()-1-i],
            };
            self.sto.insert(node.ht(), TYPENODENORMAL, 0, &mut node.bytes());
            curr_node = node.ht();
         } else {
            let node = node::TreeNode {
               child_l: siblings[siblings.len()-1-i],
               child_r: curr_node,
            };
            self.sto.insert(node.ht(), TYPENODENORMAL, 0, &mut node.bytes());
            curr_node = node.ht();
         }
      }
      curr_node
   }

   #[allow(dead_code)]
   pub fn get_value_in_pos(&self, hi: [u8;32]) -> Vec<u8> {
      let path = utils::get_path(self.num_levels, hi);
      let mut node_hash = self.root;
      for i in (0..=self.num_levels-2).rev() {
         let (t, il, node_bytes) = self.sto.get(&node_hash);
         if t == TYPENODEFINAL {
            let hi_node = utils::hash_vec(node_bytes.to_vec().split_at(il as usize).0.to_vec());
            let path_node = utils::get_path(self.num_levels, hi_node);
            let pos_diff = utils::compare_paths(&path_node, &path);
            // if pos_diff > self.num_levels {
            if pos_diff != 999 {
               return EMPTYNODEVALUE.to_vec();
            }
            return node_bytes;
         }
         let node = node::parse_node_bytes(node_bytes);
         if !path[i as usize] {
            node_hash = node.child_l;
         } else {
            node_hash = node.child_r;
         }
      }
      let (_t, _il, node_bytes) = self.sto.get(&node_hash);
      node_bytes
   }

   #[allow(dead_code)]
   pub fn generate_proof(&self, hi: [u8;32]) -> Vec<u8> {
      let mut mp: Vec<u8> = Vec::new();

      let mut empties: [u8;32] = [0;32];
      let path = utils::get_path(self.num_levels, hi);

      let mut siblings: Vec<[u8;32]> = Vec::new();
      let mut node_hash = self.root;

      for i in 0..self.num_levels {
         let (t, il, node_bytes) = self.sto.get(&node_hash);
         if t == TYPENODEFINAL {
            let real_value_in_pos = self.get_value_in_pos(hi);
            if real_value_in_pos == EMPTYNODEVALUE {
               let leaf_hi = utils::hash_vec(node_bytes.to_vec().split_at(il as usize).0.to_vec());
               let path_child = utils::get_path(self.num_levels, leaf_hi);
               let pos_diff = utils::compare_paths(&path_child, &path);
               if pos_diff == self.num_levels { // TODO use a match here, and instead of 999 return something better
                  return mp;
               }
               if pos_diff != self.num_levels-1-i {
                  let sibling = utils::calc_hash_from_leaf_and_level(pos_diff, &path_child, utils::hash_vec(node_bytes.to_vec()));
                  let mut new_siblings: Vec<[u8;32]> = Vec::new();
                  new_siblings.push(sibling);
                  new_siblings.append(&mut siblings);
                  siblings = new_siblings;
                  // set empties bit
                  let bit_pos = self.num_levels-2-pos_diff;
                  empties[(empties.len() as isize + (bit_pos as isize/8-1) as isize) as usize] |= 1 << (bit_pos%8);
               }
            }
            break
         }
         let node = node::parse_node_bytes(node_bytes);
         let sibling: [u8;32];
         if !path[self.num_levels as usize -i as usize -2] {
            node_hash = node.child_l;
            sibling = node.child_r;
         } else {
            sibling = node.child_l;
            node_hash = node.child_r;
         }
         if sibling != EMPTYNODEVALUE {
            // set empties bit
            empties[(empties.len() as isize + (i as isize/8-1) as isize) as usize] |= 1 << (i%8);
            let mut new_siblings: Vec<[u8;32]> = Vec::new();
            new_siblings.push(sibling);
            new_siblings.append(&mut siblings);
            siblings = new_siblings;
         }
      }
      mp.append(&mut empties[..].to_vec());
      for s in siblings {
         mp.append(&mut s.to_vec());
      }
      mp
   }
   #[allow(dead_code)]
   pub fn print_level(&self, parent: [u8;32], mut lvl: u32, max_level: u32) {
      use rustc_hex::ToHex;

      let mut line: String = "".to_string();
      for _ in 0..lvl {
         line += &"  ".to_string();
      }
      line += &("lvl ".to_string() + &lvl.to_string());
      line += &(" - '".to_string() + &parent.to_hex() + &"' = ".to_string());
      let (t, _, node_bytes) = self.sto.get(&parent);
      let mut node = node::TreeNode {
         child_l: EMPTYNODEVALUE,
         child_r: EMPTYNODEVALUE,
      };
      if t==TYPENODENORMAL {
         node = node::parse_node_bytes(node_bytes);
         line += &("'".to_string() + &node.child_l.to_hex() + &"' - '".to_string() + &node.child_r.to_hex() + &"'".to_string());
      } else if t == TYPENODEVALUE {
         //
      } else if t == TYPENODEFINAL {
         let hash_node_bytes = utils::hash_vec(node_bytes);
         line += &("[final] final tree node: ".to_string() + &hash_node_bytes.to_hex()+ &"\n".to_string());
         let (_, _, leaf_node_bytes) = self.sto.get(&hash_node_bytes);
         for _ in 0..lvl {
            line += "  ";
         }
         let leaf_node_string = String::from_utf8_lossy(&leaf_node_bytes);
         line += &("leaf value: ".to_string() + &leaf_node_string);
      } else {
         line += &"[EMPTY Branch]".to_string()
      }
      println!("{:?}", line);
      lvl += 1;
      if node.child_r.len()>0 && lvl<max_level && t != TYPENODEEMPTY && t != TYPENODEFINAL {
         self.print_level(node.child_l, lvl, max_level);
         self.print_level(node.child_r, lvl, max_level);
      }
   }
   pub fn print_full_tree(&self) {
      use rustc_hex::ToHex;
      self.print_level(self.root, 0, self.num_levels - 1);
      println!("root {:?}", self.root.to_hex());
   }
   pub fn print_levels_tree(&self, max_level: u32) {
      use rustc_hex::ToHex;
      self.print_level(self.root, 0, self.num_levels - 1 - max_level);
      println!("root {:?}", self.root.to_hex());
   }
}
#[allow(dead_code)]
pub fn verify_proof(root: [u8;32], mp: Vec<u8>, hi: [u8;32], ht: [u8;32], num_levels: u32) -> bool {
   let empties: Vec<u8>;
   empties = mp.split_at(32).0.to_vec();

   let mut siblings: Vec<[u8;32]> = Vec::new();
   for i in (empties.len()..mp.len()).step_by(EMPTYNODEVALUE.len()) {
      let mut sibling: [u8;32] = [0;32];
      sibling.copy_from_slice(&mp[i..i+EMPTYNODEVALUE.len()]);
      siblings.push(sibling);
   }

   let path = utils::get_path(num_levels, hi);
   let mut node_hash = ht;
   let mut sibling_used_pos = 0;

   for i in (0..=num_levels-2).rev() {
      let sibling: [u8;32];
      if (empties[empties.len()-i as usize/8-1] & (1 << (i%8))) > 0 {
         sibling = siblings[sibling_used_pos];
         sibling_used_pos += 1;
      } else {
         sibling = EMPTYNODEVALUE;
      }

      let n: node::TreeNode;
      if path[num_levels as usize - i as usize-2] {
         n = node::TreeNode {
            child_l: sibling,
            child_r: node_hash,
         }} else {
         n = node::TreeNode {
            child_l: node_hash,
            child_r: sibling,
         }}
      if node_hash == EMPTYNODEVALUE && sibling == EMPTYNODEVALUE {
         node_hash = EMPTYNODEVALUE;
      } else {
         node_hash = n.ht();
      }
   }
   if node_hash==root {
      return true;
   }
   false
}


#[cfg(test)]
   mod tests {
      use super::*;
      use rustc_hex::ToHex;

      #[test]
      fn test_hash_vec() {
         let a: Vec<u8> = From::from("test");
         let h = utils::hash_vec(a);
         assert_eq!("9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658", h.to_hex());
      }

      #[test]
      fn test_new_mt() {
         let mt: MerkleTree = new(140);
         assert_eq!(140, mt.num_levels);
         assert_eq!("0000000000000000000000000000000000000000000000000000000000000000", mt.root.to_hex());
         let (_t, _il, b) = mt.sto.get(&[0;32]);
         assert_eq!(mt.root.to_vec(), b);
      }


      #[test]
      fn test_tree_node() {
         let n = node::TreeNode {
            child_l: [1;32],
            child_r: [2;32],
         };
         assert_eq!("01010101010101010101010101010101010101010101010101010101010101010202020202020202020202020202020202020202020202020202020202020202",
                    n.bytes().to_hex());
         assert_eq!("346d8c96a2454213fcc0daff3c96ad0398148181b9fa6488f7ae2c0af5b20aa0", n.ht().to_hex());
      }

      #[test]
      fn test_add() {
         let mut mt: MerkleTree = new(140);
         assert_eq!("0000000000000000000000000000000000000000000000000000000000000000", mt.root.to_hex());
         let val = TestValue {
            bytes: vec![1,2,3,4,5],
            index_length: 3,
         };
         mt.add(&val);
         let (_t, _il, b) = mt.sto.get(&val.ht());
         assert_eq!(*val.bytes(), b);
         assert_eq!("a0e72cc948119fcb71b413cf5ada12b2b825d5133299b20a6d9325ffc3e2fbf1", mt.root.to_hex());
      }
      #[test]
      fn test_add_2() {
          let mut mt: MerkleTree = new(140);
          let val = TestValue {
              bytes: "this is a test leaf".as_bytes().to_vec(),
              index_length: 15,
          };
          assert_eq!("0000000000000000000000000000000000000000000000000000000000000000", mt.root.to_hex());
          mt.add(&val);
          let (_t, _il, b) = mt.sto.get(&val.ht());
          assert_eq!(*val.bytes(), b);
          assert_eq!("b4fdf8a653198f0e179ccb3af7e4fc09d76247f479d6cfc95cd92d6fda589f27", mt.root.to_hex());
          let val2 = TestValue {
              bytes: "this is a second test leaf".as_bytes().to_vec(),
              index_length: 15,
          };
          mt.add(&val2);
          let (_t, _il, b) = mt.sto.get(&val2.ht());
          assert_eq!(*val2.bytes(), b);
          assert_eq!("8ac95e9c8a6fbd40bb21de7895ee35f9c8f30ca029dbb0972c02344f49462e82", mt.root.to_hex());
          mt.print_full_tree();
      }
      #[test]
      fn test_generate_proof_and_verify_proof() {
          let mut mt: MerkleTree = new(140);
          let val = TestValue {
              bytes: "this is a test leaf".as_bytes().to_vec(),
              index_length: 15,
          };
          assert_eq!("0000000000000000000000000000000000000000000000000000000000000000", mt.root.to_hex());
          mt.add(&val);
          let (_t, _il, b) = mt.sto.get(&val.ht());
          assert_eq!(*val.bytes(), b);
          assert_eq!("b4fdf8a653198f0e179ccb3af7e4fc09d76247f479d6cfc95cd92d6fda589f27", mt.root.to_hex());
          let val2 = TestValue {
              bytes: "this is a second test leaf".as_bytes().to_vec(),
              index_length: 15,
          };
          mt.add(&val2);
          let (_t, _il, b) = mt.sto.get(&val2.ht());
          assert_eq!(*val2.bytes(), b);
          assert_eq!("8ac95e9c8a6fbd40bb21de7895ee35f9c8f30ca029dbb0972c02344f49462e82", mt.root.to_hex());
      
          let mp = mt.generate_proof(val2.hi());
          assert_eq!("0000000000000000000000000000000000000000000000000000000000000001fd8e1a60cdb23c0c7b2cf8462c99fafd905054dccb0ed75e7c8a7d6806749b6b", mp.to_hex());

          // verify
          let v = verify_proof(mt.root, mp, val2.hi(), val2.ht(), mt.num_levels);
          assert_eq!(true, v);
      }
      #[test]
      fn test_generate_proof_empty_leaf_and_verify_proof() {
          let mut mt: MerkleTree = new(140);
          let val = TestValue {
              bytes: "this is a test leaf".as_bytes().to_vec(),
              index_length: 15,
          };
          mt.add(&val);
          let val2 = TestValue {
              bytes: "this is a second test leaf".as_bytes().to_vec(),
              index_length: 15,
          };
          mt.add(&val2);
          assert_eq!("8ac95e9c8a6fbd40bb21de7895ee35f9c8f30ca029dbb0972c02344f49462e82", mt.root.to_hex());

         // proof of empty leaf
          let val3 = TestValue {
              bytes: "this is a third test leaf".as_bytes().to_vec(),
              index_length: 15,
          };
          let mp = mt.generate_proof(val3.hi());
          assert_eq!("000000000000000000000000000000000000000000000000000000000000000389741fa23da77c259781ad8f4331a5a7d793eef1db7e5200ddfc8e5f5ca7ce2bfd8e1a60cdb23c0c7b2cf8462c99fafd905054dccb0ed75e7c8a7d6806749b6b", mp.to_hex());

          // verify that is a proof of an empty leaf (EMPTYNODEVALUE)
          let v = verify_proof(mt.root, mp, val3.hi(), EMPTYNODEVALUE, mt.num_levels);
          assert_eq!(true, v);
      }
      #[test]
      fn test_harcoded_proofs_of_existing_leaf() {
         // check proof of value in leaf
         let mut root: [u8;32] = [0;32];
         root.copy_from_slice(&hex::decode("7d7c5e8f4b3bf434f3d9d223359c4415e2764dd38de2e025fbf986e976a7ed3d").unwrap());
         let mp = hex::decode("0000000000000000000000000000000000000000000000000000000000000002d45aada6eec346222eaa6b5d3a9260e08c9b62fcf63c72bc05df284de07e6a52").unwrap();
         let mut hi: [u8;32] = [0;32];
         hi.copy_from_slice(&hex::decode("786677808ba77bdd9090a969f1ef2cbd1ac5aecd9e654f340500159219106878").unwrap());
         let mut ht: [u8;32] = [0;32];
         ht.copy_from_slice(&hex::decode("786677808ba77bdd9090a969f1ef2cbd1ac5aecd9e654f340500159219106878").unwrap());
          let v = verify_proof(root, mp, hi, ht, 140);
          assert_eq!(true, v);
      }
      #[test]
      fn test_harcoded_proofs_of_empty_leaf() {
         // check proof of value in leaf
         let mut root: [u8;32] = [0;32];
         root.copy_from_slice(&hex::decode("8f021d00c39dcd768974ddfe0d21f5d13f7215bea28db1f1cb29842b111332e7").unwrap());
         let mp = hex::decode("0000000000000000000000000000000000000000000000000000000000000004bf8e980d2ed328ae97f65c30c25520aeb53ff837579e392ea1464934c7c1feb9").unwrap();
         let mut hi: [u8;32] = [0;32];
         hi.copy_from_slice(&hex::decode("a69792a4cff51f40b7a1f7ae596c6ded4aba241646a47538898f17f2a8dff647").unwrap());
          let v = verify_proof(root, mp, hi, EMPTYNODEVALUE, 140);
          assert_eq!(true, v);
      }
   }
