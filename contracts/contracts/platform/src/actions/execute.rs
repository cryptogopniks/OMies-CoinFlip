use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdResult,
    Storage, Uint128, WasmMsg,
};

use cf_base::{
    converters::address_to_salt,
    error::ContractError,
    hash_generator::types::Hash,
    platform::{
        state::{
            APP_INFO, CONFIG, FLIP_COOLDOWN, IS_PAUSED, NORMALIZED_DECIMAL, TRANSFER_ADMIN_STATE,
            TRANSFER_ADMIN_TIMEOUT, USERS,
        },
        types::{Config, Range, TransferAdminState, UserInfo},
    },
    utils::{check_authorization, check_funds, AuthType, FundsType},
};
use hashing_helper::base::calc_hash_bytes;

use crate::helpers::{check_pause_state, get_random_weight};

//    Flip(Side),

//    Claim {},

pub fn try_deposit(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;
    let denom = asset_info.try_get_native()?;
    let config = CONFIG.load(deps.storage)?;

    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Admin,
    )?;

    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    // check fund denom
    if denom != config.denom {
        Err(ContractError::WrongAssetType)?;
    }

    APP_INFO.update(deps.storage, |mut x| -> StdResult<_> {
        x.deposited += asset_amount;
        x.balance += asset_amount;
        Ok(x)
    })?;

    Ok(Response::new().add_attribute("action", "try_deposit"))
}

pub fn try_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;

    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Admin,
    )?;

    if amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    APP_INFO.update(deps.storage, |mut x| -> StdResult<_> {
        if amount > x.balance {
            Err(ContractError::NotEnoughLiquidity)?;
        }

        x.deposited -= std::cmp::min(amount, x.deposited);
        x.balance -= amount;
        Ok(x)
    })?;

    let msg = BankMsg::Send {
        to_address: recipient
            .map(|x| deps.api.addr_validate(&x))
            .transpose()?
            .unwrap_or(sender_address)
            .to_string(),
        amount: coins(amount.u128(), config.denom),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_withdraw"))
}

pub fn try_accept_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    let block_time = env.block.time.seconds();
    let TransferAdminState {
        new_admin,
        deadline,
    } = TRANSFER_ADMIN_STATE.load(deps.storage)?;

    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Specified {
            allowlist: vec![Some(new_admin)],
        },
    )?;

    if block_time >= deadline {
        Err(ContractError::TransferAdminDeadline)?;
    }

    CONFIG.update(deps.storage, |mut x| -> StdResult<_> {
        x.admin = sender_address;
        Ok(x)
    })?;

    TRANSFER_ADMIN_STATE.update(deps.storage, |mut x| -> StdResult<_> {
        x.deadline = block_time;
        Ok(x)
    })?;

    Ok(Response::new().add_attribute("action", "try_accept_admin_role"))
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    worker: Option<String>,
    bet: Option<Range>,
    denom: Option<String>,
    platform_fee: Option<Decimal>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut is_config_updated = false;

    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Admin,
    )?;

    if let Some(x) = admin {
        let block_time = env.block.time.seconds();
        let new_admin = &deps.api.addr_validate(&x)?;

        TRANSFER_ADMIN_STATE.save(
            deps.storage,
            &TransferAdminState {
                new_admin: new_admin.to_owned(),
                deadline: block_time + TRANSFER_ADMIN_TIMEOUT,
            },
        )?;

        is_config_updated = true;
    }

    if let Some(x) = worker {
        config.worker = Some(deps.api.addr_validate(&x)?);
        is_config_updated = true;
    }

    if let Some(x) = bet {
        config.bet = x;
        is_config_updated = true;
    }

    if let Some(x) = denom {
        config.denom = x;
        is_config_updated = true;
    }

    if let Some(x) = platform_fee {
        config.platform_fee = x;
        is_config_updated = true;
    }

    // don't allow empty messages
    if !is_config_updated {
        Err(ContractError::NoParameters)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "try_update_config"))
}

pub fn try_pause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    check_authorization(&sender_address, &admin, &worker, AuthType::AdminOrWorker)?;

    IS_PAUSED.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("action", "try_pause"))
}

pub fn try_unpause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let Config { admin, worker, .. } = CONFIG.load(deps.storage)?;
    check_authorization(&sender_address, &admin, &worker, AuthType::Admin)?;

    IS_PAUSED.save(deps.storage, &false)?;

    Ok(Response::new().add_attribute("action", "try_unpause"))
}
