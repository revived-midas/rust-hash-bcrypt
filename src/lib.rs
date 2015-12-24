//! Easily hash and verify passwords using bcrypt
//!

#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

#[macro_use]
extern crate lazy_static;
extern crate crypto;
extern crate rand;
extern crate rustc_serialize;

use std::io;

use crypto::bcrypt::bcrypt;
use crypto::util::fixed_time_eq;
use rand::{Rng, OsRng};

mod b64;


// Cost constants
static MIN_COST: u32 = 4;
static MAX_COST: u32 = 31;
static DEFAULT_COST: u32 = 12;


#[derive(Debug, PartialEq)]
struct HashParts {
 cost: u32,
 salt: String,
 hash: String,
}

impl HashParts {
    fn format(self) -> String {
        // Cost need to have a length of 2 so padding with a 0 if cost < 10
        format!("$2y${:02}${}{}", self.cost, self.salt, self.hash)
    }
}

fn _hash(password: &str, cost: u32, salt: &[u8]) -> HashParts {
    // Output is 24
    let mut output = [0u8; 24];
    // We only consider the first 72 chars
    let password_bytes: &[u8] = password.as_ref();
    let pass = if password_bytes.len() > 72 {
        &password_bytes[..72]
    } else {
        password_bytes
    };
    // Passwords need to be null terminated
    let mut vec = Vec::new();
    vec.extend(pass);
    vec.push(0);

    bcrypt(cost, &salt, &vec, &mut output);

    HashParts {
        cost: cost,
        salt: b64::encode(&salt),
        hash: b64::encode(&output[..23])
    }
}

pub fn hash(password: &str, cost: u32) -> io::Result<String> {
    // TODO: check cost value and error if necessary
    let salt = {
        let mut s = [0u8; 16];
        let mut rng = try!(OsRng::new());
        rng.fill_bytes(&mut s);
        s
    };
    let hash_parts = _hash(password, cost, &salt);

    Ok(hash_parts.format())
}

fn split_hash(hash: &str) -> HashParts {
    let mut parts = HashParts {
        cost: 0,
        salt: "".to_owned(),
        hash: "".to_owned()
    };

    for (i, part) in hash.split('$').enumerate() {
        match i {
            0 => (),
            1 => match part {
                "2y" | "2b" | "2a" => (),
                _ => ()
            },
            2 => {
                if let Ok(c) = part.parse::<u32>() {
                    parts.cost = c;
                } else {
                    ()
                }
            },
            3 => {
                println!("{:?}, {}", part, part.len());
                if part.len() == 53 {
                    parts.salt = part[..22].chars().collect();
                    parts.hash = part[22..].chars().collect();
                }
            },
            _ => ()
        }
    }

    parts
}

pub fn verify(password: &str, hash: &str) -> Result<bool, ()> {
    let parts = split_hash(hash);
    println!("{:?}", parts);
    // TODO: handle invalid hash parts
    let salt = b64::decode(&parts.salt);
    let generated = _hash(password, parts.cost, &salt);
    println!("{:?}", generated);

    // Hashes should be the same given same salt and round number
    Ok(fixed_time_eq(&b64::decode(&parts.hash), &b64::decode(&generated.hash)))
}


#[cfg(test)]
mod tests {
    use super::{hash, verify, HashParts, split_hash};

    #[test]
    fn can_split_hash() {
        let hash = "$2y$12$L6Bc/AlTQHyd9liGgGEZyOFLPHNgyxeEPfgYfBCVxJ7JIlwxyVU3u";
        let output = split_hash(hash);
        let expected = HashParts {
            cost: 12,
            salt: "L6Bc/AlTQHyd9liGgGEZyO".to_owned(),
            hash: "FLPHNgyxeEPfgYfBCVxJ7JIlwxyVU3u".to_owned()
        };
        assert_eq!(output, expected);
    }

    #[test]
    fn can_verify_hash_generated_from_some_online_tool() {
        let hash = "$2a$04$UuTkLRZZ6QofpDOlMz32MuuxEHA43WOemOYHPz6.SjsVsyO1tDU96";
        assert!(verify("password", hash).unwrap() == true);
    }

    #[test]
    fn can_verify_hash_generated_from_python() {
        let hash = "$2b$04$EGdrhbKUv8Oc9vGiXX0HQOxSg445d458Muh7DAHskb6QbtCvdxcie";
        assert!(verify("correctbatteryhorsestapler", hash).unwrap() == true);
    }

    #[test]
    fn can_verify_hash_generated_from_node() {
        let hash = "$2a$04$n4Uy0eSnMfvnESYL.bLwuuj0U/ETSsoTpRT9GVk5bektyVVa5xnIi";
        assert!(verify("correctbatteryhorsestapler", hash).unwrap() == true);
    }

    #[test]
    fn a_wrong_password_is_false() {
        let hash = "$2b$04$EGdrhbKUv8Oc9vGiXX0HQOxSg445d458Muh7DAHskb6QbtCvdxcie";
        assert!(verify("wrong", hash).unwrap() == false);
    }

    #[test]
    fn can_verify_own_generated() {
        let hashed = hash("hunter2", 4).unwrap();
        assert_eq!(true, verify("hunter2", &hashed).unwrap());
    }
}
