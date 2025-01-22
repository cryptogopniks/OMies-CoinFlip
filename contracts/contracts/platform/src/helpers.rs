use cosmwasm_std::{Addr, Decimal, Env, StdResult, Storage};

use cf_base::{
    converters::address_to_salt, error::ContractError, hash_generator::types::Hash,
    platform::state::IS_PAUSED,
};
use hashing_helper::base::calc_hash_bytes;

/// user actions are disabled when the contract is paused
pub fn check_pause_state(storage: &dyn Storage) -> StdResult<()> {
    if IS_PAUSED.load(storage)? {
        Err(ContractError::ContractIsPaused)?;
    }

    Ok(())
}

pub fn get_random_weight(
    env: &Env,
    sender_address: &Addr,
    previous_weight: &Decimal,
) -> StdResult<Decimal> {
    let password = &format!("{}{}", previous_weight, env.block.time.nanos());
    let salt = &address_to_salt(sender_address);
    let hash_bytes = calc_hash_bytes(password, salt)?;

    Ok(Hash::from(hash_bytes).to_norm_dec())
}
