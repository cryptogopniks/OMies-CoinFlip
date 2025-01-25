use cosmwasm_std::{Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

use cf_base::platform::{
    msg::UserListRespItem,
    state::{APP_INFO, CONFIG, USERS},
    types::{AppInfo, Config, UserInfo},
};

use crate::helpers::calc_available_to_withdraw;

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_app_info(deps: Deps, _env: Env) -> StdResult<AppInfo> {
    APP_INFO.load(deps.storage)
}

pub fn query_available_to_withdraw(deps: Deps, _env: Env) -> StdResult<Uint128> {
    let x = APP_INFO.load(deps.storage)?;
    Ok(calc_available_to_withdraw(x.deposited, x.revenue.current))
}

pub fn query_user(deps: Deps, _env: Env, address: String) -> StdResult<UserInfo> {
    Ok(USERS
        .load(deps.storage, &deps.api.addr_validate(&address)?)
        .unwrap_or_default())
}

pub fn query_user_list(
    deps: Deps,
    _env: Env,
    amount: u32,
    start_after: Option<String>,
) -> StdResult<Vec<UserListRespItem>> {
    let binding;
    let start_bound = match start_after {
        Some(addr) => {
            binding = deps.api.addr_validate(&addr)?;
            Some(Bound::exclusive(&binding))
        }
        None => None,
    };

    Ok(USERS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(amount as usize)
        .map(|x| {
            let (address, info) = x.unwrap();
            UserListRespItem { address, info }
        })
        .collect())
}
