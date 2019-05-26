use node;
use tiny_keccak::Keccak;

pub fn hash_vec(b: Vec<u8>) -> [u8; 32] {
    // let mut sha3 = Keccak::new_sha3_256();
    let mut sha3 = Keccak::new_keccak256();
    sha3.update(&b);
    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    res
}

#[allow(dead_code)]
pub fn get_path(num_levels: u32, hi: [u8;32]) -> Vec<bool> {
    let mut path = Vec::new();
    for i in (0..num_levels as usize-1).rev() {
        path.push((hi[hi.len()-i/8-1] & (1 << (i%8))) > 0);
    }
    path
}

#[allow(dead_code)]
pub fn calc_hash_from_leaf_and_level(until_level: u32, path: &[bool], leaf_hash: [u8;32]) -> [u8;32] {
    let mut node_curr_lvl = leaf_hash;
    for i in 0..until_level {
        if path[i as usize] {
            let node = node::TreeNode {
                child_l: ::EMPTYNODEVALUE,
                child_r: node_curr_lvl,
            };
            node_curr_lvl = node.ht();
        } else {
            let node = node::TreeNode {
                child_l: node_curr_lvl,
                child_r: ::EMPTYNODEVALUE,
            };
            node_curr_lvl = node.ht();
        }
    }
    node_curr_lvl
}

pub fn cut_path(path: &[bool], i: usize) -> Vec<bool> {
    let mut path_res: Vec<bool> = Vec::new();
    for (j, path_elem) in path.iter().enumerate() {
        if j >= i {
            path_res.push(*path_elem);
        }
    }
    path_res
}

pub fn compare_paths(a: &[bool], b: &[bool]) -> u32 {
    for i in (0..a.len()).rev() {
        if a[i] != b[i] {
            return i as u32;
        }
    }
    999
}

pub fn get_empties_between_i_and_pos(i: u32, pos: u32) -> Vec<[u8;32]> {
    let mut sibl: Vec<[u8;32]> = Vec::new();
    for _j in (pos..i).rev() {
        sibl.push(::EMPTYNODEVALUE);
    }
    sibl.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::ToHex;

    #[test]
    fn test_hash_vec() {
        let a: Vec<u8> = From::from("test");
        assert_eq!("74657374", a.to_hex());
        let h = hash_vec(a);
        assert_eq!("9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658", h.to_hex());
    }
}
