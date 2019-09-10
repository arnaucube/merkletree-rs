use super::constants;
use super::utils;

pub struct TreeNode {
    pub child_l: [u8; 32],
    pub child_r: [u8; 32],
}

impl TreeNode {
    pub fn bytes(&self) -> Vec<u8> {
        concatenate_arrays(&self.child_l, &self.child_r)
    }
    pub fn ht(&self) -> [u8; 32] {
        utils::hash_vec(self.bytes())
    }
}

fn concatenate_arrays<T: Clone>(x: &[T], y: &[T]) -> Vec<T> {
    let mut concat = x.to_vec();
    concat.extend_from_slice(y);

    concat
}

pub fn parse_node_bytes(b: Vec<u8>) -> TreeNode {
    if b == constants::EMPTYNODEVALUE {
        let n = TreeNode {
            child_l: constants::EMPTYNODEVALUE,
            child_r: constants::EMPTYNODEVALUE,
        };
        return n;
    }
    let child_l = &b[0..32];
    let child_r = &b[32..];
    TreeNode {
        child_l: *array_ref!(child_l, 0, 32),
        child_r: *array_ref!(child_r, 0, 32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::ToHex;

    #[test]
    fn test_hash_vec() {
        let n = TreeNode {
            child_l: constants::EMPTYNODEVALUE,
            child_r: constants::EMPTYNODEVALUE,
        };
        assert_eq!(
            "ad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5",
            n.ht().to_hex()
        )
    }
}
