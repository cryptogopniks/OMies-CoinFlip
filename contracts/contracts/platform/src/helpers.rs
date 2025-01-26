use cosmwasm_std::{Addr, Decimal, Env, Int256, StdResult, Storage, Uint128};

use cf_base::{
    converters::{address_to_salt, str_to_dec},
    error::ContractError,
    hash_generator::types::Hash,
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

pub fn calc_required_to_deposit(balance: Uint128, total_unclaimed: Uint128) -> Uint128 {
    if balance >= total_unclaimed {
        Uint128::zero()
    } else {
        total_unclaimed - balance
    }
}

pub fn calc_available_to_withdraw(deposited: Uint128, revenue_current: Int256) -> Uint128 {
    let available_to_withdraw = Int256::from(deposited) + revenue_current;

    if available_to_withdraw.is_negative() {
        Uint128::zero()
    } else {
        str_to_dec(&available_to_withdraw.to_string()).to_uint_floor()
    }
}
