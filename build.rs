use blake2b_rs::{Blake2b, Blake2bBuilder};

use std::{
    fs::{self, File},
    io::{Read, Result, Write},
};

const PATH_PREFIX: &str = "specs/cells/";
const BUF_SIZE: usize = 8 * 1024;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

const BINARIES: &[&str] = &["dckb", "dao_lock"];

fn gen_const(code_hashes: &[(&str, [u8; 32])]) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("c/const.h")?;
    f.write_all(
        "/* Do not touch this file! This file is auto generated by build.rs */\n\
                #ifndef DCKB_CONST_H\n\
                #define DCKB_CONST_H\n"
            .as_bytes(),
    )?;
    // write code hashes
    for (const_name, value) in code_hashes {
        let v = format!(
            "{{{}}}",
            value
                .iter()
                .map(|i| format!("{}", i))
                .collect::<Vec<_>>()
                .join(" ,")
        );
        f.write_all(
            &format!(
                "const uint8_t {const_name}[] = {value};\n",
                const_name = const_name,
                value = v,
            )
            .into_bytes(),
        )?;
    }
    f.write_all("#endif".as_bytes())?;
    Ok(())
}

fn read_hash(name: &str) -> [u8; 32] {
    let mut buf = [0u8; BUF_SIZE];
    let path = format!("{}{}", PATH_PREFIX, name);
    let mut fd = File::open(&path).expect("open file");
    let mut blake2b = new_blake2b();
    loop {
        let read_bytes = fd.read(&mut buf).expect("read file");
        if read_bytes > 0 {
            blake2b.update(&buf[..read_bytes]);
        } else {
            break;
        }
    }
    let mut hash = [0u8; 32];
    blake2b.finalize(&mut hash);
    hash
}

fn main() {
    let dao_lock_code_hash = read_hash("dao_lock");
    gen_const(&[
        ("DAO_LOCK_CODE_HASH", dao_lock_code_hash),
        ("CUSTODIAN_LOCK_CODE_HASH", [0u8; 32]),
    ])
    .expect("gen constants");
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("code_hashes.txt")
        .expect("open");
    for name in BINARIES {
        let hash = read_hash(name);
        let actual_hash = faster_hex::hex_string(&hash).expect("hex");
        f.write_all(&format!("{} -> {}\n", name, actual_hash).into_bytes())
            .expect("write hash");
    }
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}
