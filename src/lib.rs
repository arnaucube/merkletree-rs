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

impl MerkleTree {
   pub fn add(&mut self, v: &TestValue) {
      #![allow(unused_variables)]
      #[allow(dead_code)]

      // println!("adding value: {:?}", v.bytes());
      // add the leaf that we are adding
      self.sto.insert(utils::hash_vec(v.bytes().to_vec()), TYPENODEVALUE, v.index_length(), &mut v.bytes().to_vec());

      let index = v.index_length() as usize;
      let hi = utils::hash_vec(v.bytes()[..index].to_vec());
      let ht = utils::hash_vec(v.bytes().to_vec());
      let path = utils::get_path(self.num_levels, hi);
      let mut siblings: Vec<[u8;32]> = Vec::new();


      let mut node_hash = self.root;

      for i in (0..self.num_levels-1).rev() {
         // get node
         // let (t, il, node_bytes) = self.sto.get(&utils::hash_vec(node_hash.to_vec()));
         let (t, il, node_bytes) = self.sto.get(&node_hash);
         if t == TYPENODEFINAL {
            let hi_child = utils::hash_vec(v.bytes().to_vec().split_off(il as usize));
            let path_child = utils::get_path(self.num_levels, hi_child);
            let pos_diff = utils::compare_paths(path_child.clone(), path.clone());
            if pos_diff == 999 { // TODO use a match here, and instead of 999 return something better
               println!("compare paths err");
               return;
            }
            let final_node_1_hash = utils::calc_hash_from_leaf_and_level(pos_diff, path_child.clone(), utils::hash_vec(node_bytes.to_vec()));
            self.sto.insert(final_node_1_hash, TYPENODEFINAL, il, &mut node_bytes.to_vec());
            let final_node_2_hash = utils::calc_hash_from_leaf_and_level(pos_diff, path.clone(), utils::hash_vec(v.bytes().to_vec()));
            self.sto.insert(final_node_2_hash, TYPENODEFINAL, v.index_length(), &mut v.bytes().to_vec());

            // parent node
            let parent_node: node::TreeNode;
            if path[pos_diff as usize] {
               parent_node = node::TreeNode {
                  child_l: final_node_1_hash,
                  child_r: final_node_2_hash,
               };
            } else {
               parent_node = node::TreeNode {
                  child_l: final_node_2_hash,
                  child_r: final_node_1_hash,
               };
            }
            let empties = utils::get_empties_between_i_and_pos(i, pos_diff+1);
            for j in 0..empties.len() {
               siblings.push(empties[j]);
            }
            let path_from_pos_diff = utils::cut_path(path.clone(), (pos_diff +1) as usize);
            self.root = self.replace_leaf(path_from_pos_diff, siblings.clone(), parent_node.ht(), TYPENODENORMAL, 0, &mut parent_node.bytes().to_vec());
            return;
         }

         let node = node::parse_node_bytes(node_bytes);

         let sibling: [u8;32];
         if path.clone().into_iter().nth(i as usize).unwrap() {
            node_hash = node.child_l;
            sibling = node.child_r;
         } else {
            sibling = node.child_l;
            node_hash = node.child_r;
         }
         siblings.push(*array_ref!(sibling, 0, 32));
         if node_hash == EMPTYNODEVALUE {
            if i==self.num_levels-2 && siblings[siblings.len()-1]==EMPTYNODEVALUE {

               let final_node_hash = utils::calc_hash_from_leaf_and_level(i+1, path.clone(), utils::hash_vec(v.bytes().to_vec()));
               self.sto.insert(final_node_hash, TYPENODEFINAL, v.index_length(), &mut v.bytes().to_vec());
               self.root = final_node_hash;
               return;
            }
            let final_node_hash = utils::calc_hash_from_leaf_and_level(i, path.clone(), utils::hash_vec(v.bytes().to_vec()));
            let path_from_i = utils::cut_path(path.clone(), i as usize);
            self.root = self.replace_leaf(path_from_i, siblings.clone(), final_node_hash, TYPENODEFINAL, v.index_length(), &mut v.bytes().to_vec());
            return;
         }
      }
      self.root = self.replace_leaf(path, siblings, utils::hash_vec(v.bytes().to_vec()), TYPENODEVALUE, v.index_length(), &mut v.bytes().to_vec());
      }

      #[allow(dead_code)]
      pub fn replace_leaf(&mut self, path: Vec<bool>, siblings: Vec<[u8;32]>, leaf_hash: [u8;32], node_type: u8, index_length: u32, leaf_value: &mut Vec<u8>) -> [u8;32] {
         self.sto.insert(leaf_hash, node_type, index_length, leaf_value);
         let mut curr_node = leaf_hash;

         for i in 0..siblings.len() {
            if path.clone().into_iter().nth(i as usize).unwrap() {
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
         let (_t, _il, b) = mt.sto.get(&utils::hash_vec(val.bytes().to_vec()));
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
         let (_t, _il, b) = mt.sto.get(&utils::hash_vec(val.bytes().to_vec()));
          assert_eq!(*val.bytes(), b);
          assert_eq!("b4fdf8a653198f0e179ccb3af7e4fc09d76247f479d6cfc95cd92d6fda589f27", mt.root.to_hex());
      }
   }
