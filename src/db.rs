extern crate rusty_leveldb;

use self::rusty_leveldb::DB;

use super::constants;

pub struct Db {
    storage: DB,
}

impl Db {
    pub fn new(path: String, in_memory: bool) -> Db {
        let opt: rusty_leveldb::Options;
        if in_memory {
            opt = rusty_leveldb::in_memory();
        } else {
            opt = Default::default();
        }
        let database = DB::open(path, opt).unwrap();
        Db { storage: database }
    }
    pub fn insert(&mut self, k: [u8; 32], t: u8, il: u32, b: Vec<u8>) {
        let mut v: Vec<u8>;
        v = [t].to_vec();
        let il_bytes = il.to_le_bytes();
        v.extend(il_bytes.to_vec()); // il_bytes are [u8;4] (4 bytes)
        v.extend(&b);
        self.storage.put(&k[..], &v[..]).unwrap();
    }
    pub fn get(&mut self, k: &[u8; 32]) -> (u8, u32, Vec<u8>) {
        if k.to_vec() == constants::EMPTYNODEVALUE.to_vec() {
            return (0, 0, constants::EMPTYNODEVALUE.to_vec());
        }
        match self.storage.get(k) {
            Some(x) => {
                let t = x[0];
                let il_bytes: [u8; 4] = [x[1], x[2], x[3], x[4]];
                let il = u32::from_le_bytes(il_bytes);
                let b = &x[5..];
                (t, il, b.to_vec())
            }
            None => (
                constants::TYPENODEEMPTY,
                0,
                constants::EMPTYNODEVALUE.to_vec(),
            ),
        }
    }
}
