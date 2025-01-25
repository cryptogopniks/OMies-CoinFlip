use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use cf_base::{
    converters::str_to_dec,
    error::ContractError,
    platform::{
        msg::InstantiateMsg,
        state::{
            APP_INFO, BET_MAX, BET_MIN, CONFIG, CONTRACT_NAME, DENOM, IS_PAUSED,
            NORMALIZED_DECIMAL, PLATFORM_FEE, SEED, TRANSFER_ADMIN_STATE,
        },
        types::{AppInfo, Config, Range, TransferAdminState},
    },
};

use crate::helpers::get_random_weight;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn try_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let sender = &info.sender;
    let block_time = env.block.time.seconds();

    IS_PAUSED.save(deps.storage, &false)?;
    TRANSFER_ADMIN_STATE.save(
        deps.storage,
        &TransferAdminState {
            new_admin: sender.clone(),
            deadline: block_time,
        },
    )?;

    CONFIG.save(
        deps.storage,
        &Config {
            admin: sender.to_owned(),
            worker: msg
                .worker
                .map(|x| deps.api.addr_validate(&x))
                .transpose()
                .unwrap_or(Some(sender.to_owned())),
            bet: msg.bet.unwrap_or(Range::new(BET_MIN, BET_MAX)),
            denom: String::from(DENOM),
            platform_fee: msg.platform_fee.unwrap_or(str_to_dec(PLATFORM_FEE)),
        },
    )?;

    NORMALIZED_DECIMAL.save(
        deps.storage,
        &get_random_weight(&env, sender, &str_to_dec(SEED))?,
    )?;
    APP_INFO.save(deps.storage, &AppInfo::default())?;

    Ok(Response::new().add_attribute("action", "try_instantiate"))
}
