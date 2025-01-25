use cosmwasm_std::{StdError, StdResult};

use argon2::{Algorithm, Argon2, ParamsBuilder, Version};

pub const ENC_KEY_LEN: usize = 32;

/// Accepts `password` as string of letters and numbers and `salt` as string of letters and numbers
pub fn calc_hash_bytes(password: &str, salt: &str) -> StdResult<[u8; ENC_KEY_LEN]> {
    const MEMORY_SIZE: u32 = 64;
    const NUMBER_OF_ITERATIONS: u32 = 4;
    const DEGREE_OF_PARALLELISM: u32 = 1;
    const ALGORITHM: Algorithm = Algorithm::Argon2id;
    const VERSION: Version = Version::V0x10;

    let params = ParamsBuilder::new()
        .m_cost(MEMORY_SIZE)
        .t_cost(NUMBER_OF_ITERATIONS)
        .p_cost(DEGREE_OF_PARALLELISM)
        .output_len(ENC_KEY_LEN)
        .build()
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    let ctx = Argon2::new(ALGORITHM, VERSION, params);

    let mut out = [0; ENC_KEY_LEN];
    ctx.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut out)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    Ok(out)
}
