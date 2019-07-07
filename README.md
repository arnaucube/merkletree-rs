# merkletree-rs [![Build Status](https://travis-ci.org/arnaucube/merkletree-rs.svg?branch=master)](https://travis-ci.org/arnaucube/merkletree-rs)
Sparse MerkleTree implementation in Rust.

The MerkleTree is optimized in the design and concepts, to have a faster and lighter MerkleTree, maintaining compatibility with a non optimized MerkleTree. In this way, the MerkleRoot of the optimized MerkleTree will be the same that the MerkleRoot of the non optimized MerkleTree.

Compatible with the Go version: https://github.com/arnaucube/go-merkletree


## Usage
Create new tree:
```rust
let mut mt: MerkleTree = new(140);
```

Add value to leaf:
```rust
let val = TestValue {
  bytes: "this is a test leaf".as_bytes().to_vec(),
  index_length: 15,
};
mt.add(&val);
```

Get proof:
```rust
let mp = mt.generate_proof(val.hi());
```

Verify proof:
```rust
// check if the value exist
let v = verify_proof(mt.root, mp, val.hi(), val.ht(), mt.num_levels);

// check if the don't value exist
let v = verify_proof(mt.root, mp, val.hi(), EMPTYNODEVALUE, mt.num_levels);
```
