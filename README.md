# merkletree-rs [![Crates.io](https://img.shields.io/crates/v/merkletree-rs.svg)](https://crates.io/crates/merkletree-rs) [![Build Status](https://travis-ci.org/arnaucube/merkletree-rs.svg?branch=master)](https://travis-ci.org/arnaucube/merkletree-rs)
Sparse MerkleTree implementation in Rust.

The MerkleTree is optimized in the design and concepts, to have a faster and lighter MerkleTree, maintaining compatibility with a non optimized MerkleTree. In this way, the MerkleRoot of the optimized MerkleTree will be the same that the MerkleRoot of the non optimized MerkleTree.

Compatible with the Go version: https://github.com/arnaucube/go-merkletree


## Usage
Import lib:
```rust
extern crate merkletree_rs;
use merkletree_rs::{db, MerkleTree, TestValue, Value};
```

Create new tree:
```rust
// to build the storage, the first parameter is the path and the second parameter specifies if wants to use a in_memory database or a directory of the filesystem
let mut sto = db::Db::new("test".to_string(), true);
let mut mt = MerkleTree::new(&mut sto, 140 as u32);
```

Add value to leaf:
```rust
let val: TestValue = TestValue {
    bytes: "this is a test leaf".as_bytes().to_vec(),
    index_length: 15,
};
mt.add(&val).unwrap();
```

Get proof:
```rust
let mp = mt.generate_proof(val.hi());
println!("{:?}", mp);
```

Verify proof:
```rust
// check if the value exist
let v =
    merkletree_rs::verify_proof(mt.get_root(), &mp, val.hi(), val.ht(), mt.get_num_levels());
println!("{:?}", v);

// check if the don't value exist (in that case, the 'ht' will be an empty value)
let v = merkletree_rs::verify_proof(
    mt.get_root(),
    &mp,
    val.hi(),
    merkletree_rs::constants::EMPTYNODEVALUE,
    mt.get_num_levels(),
);
println!("{:?}", v);
```

Print current MerkleRoot:
```rust
println!("{:?}", mt.get_root());
```
