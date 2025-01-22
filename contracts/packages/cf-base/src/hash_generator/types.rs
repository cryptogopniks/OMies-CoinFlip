use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, StdResult};

use hashing_helper::base::ENC_KEY_LEN;

use crate::converters::{
    hash_bytes_to_norm_dec, str_to_u8_vec, u8_vec_to_hash_bytes, u8_vec_to_str,
};

#[cw_serde]
pub struct Hash {
    bytes: [u8; ENC_KEY_LEN],
}

impl Hash {
    pub fn parse(hash_str: &str) -> StdResult<Self> {
        u8_vec_to_hash_bytes(&str_to_u8_vec(hash_str)).map(|bytes| Self { bytes })
    }

    pub fn to_norm_dec(&self) -> Decimal {
        hash_bytes_to_norm_dec(&self.bytes)
    }
}

impl From<[u8; ENC_KEY_LEN]> for Hash {
    fn from(bytes: [u8; ENC_KEY_LEN]) -> Self {
        Self { bytes }
    }
}

impl From<Hash> for [u8; ENC_KEY_LEN] {
    fn from(hash: Hash) -> Self {
        hash.bytes
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", u8_vec_to_str(&self.bytes))
    }
}
