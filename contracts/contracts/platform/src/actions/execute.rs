use cosmwasm_std::{Decimal, DepsMut, Env, Int256, MessageInfo, Response, StdResult, Uint128};

use cf_base::{
    assets::Token,
    error::ContractError,
    platform::{
        state::{
            APP_INFO, CONFIG, FLIP_COOLDOWN, IS_PAUSED, NORMALIZED_DECIMAL, TRANSFER_ADMIN_STATE,
            TRANSFER_ADMIN_TIMEOUT, USERS,
        },
        types::{Config, Range, Side, TransferAdminState},
    },
    utils::{check_authorization, check_funds, get_transfer_msg, AuthType, FundsType},
};

use crate::helpers::{calc_available_to_withdraw, check_pause_state, get_random_weight};

pub fn try_flip(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    side: Side,
) -> Result<Response, ContractError> {
    let mut response = Response::new().add_attribute("action", "try_flip");
    check_pause_state(deps.storage)?;
    let (sender_address, asset_amount, asset_info) = check_funds(
        deps.as_ref(),
        &info,
        FundsType::Single {
            sender: None,
            amount: None,
        },
    )?;
    let block_time = env.block.time.seconds();
    let normalized_decimal = NORMALIZED_DECIMAL.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let mut app_info = APP_INFO.load(deps.storage)?;
    let mut user = USERS
        .load(deps.storage, &sender_address)
        .unwrap_or_default();

    // don't allow to flip multiple coins in single tx
    if block_time < user.last_flip_date + FLIP_COOLDOWN {
        Err(ContractError::MultipleFlipsPerTx)?;
    }

    // check fund amount
    if asset_amount.is_zero() {
        Err(ContractError::ZeroAmount)?;
    }

    config.bet.validate(asset_amount)?;

    // check fund denom
    if asset_info.try_get_native()? != config.denom {
        Err(ContractError::WrongAssetType)?;
    }

    let random_weight = get_random_weight(&env, &sender_address, &normalized_decimal)?;
    let is_winner = side.is_winner(random_weight, config.platform_fee);
    let prize = if is_winner {
        Uint128::new(2) * asset_amount
    } else {
        Uint128::zero()
    };

    app_info.revenue.total += Int256::from(asset_amount);
    app_info.revenue.current += Int256::from(asset_amount);
    app_info.balance += asset_amount;

    if is_winner {
        app_info.revenue.total -= Int256::from(prize);
        app_info.revenue.current -= Int256::from(prize);

        if app_info.balance >= prize {
            app_info.balance -= prize;
            response = response.add_message(get_transfer_msg(&sender_address, prize, &asset_info)?);
        } else {
            app_info.user_unclaimed += prize;
            user.unclaimed += prize;
        }

        app_info.user_stats.wins.increase(prize);
        user.stats.wins.increase(prize);
    }

    app_info.user_stats.bets.increase(asset_amount);
    app_info.update_average_fee();

    user.stats.bets.increase(asset_amount);
    user.update_roi();
    user.last_flip_date = block_time;

    NORMALIZED_DECIMAL.save(deps.storage, &random_weight)?;
    APP_INFO.save(deps.storage, &app_info)?;
    USERS.save(deps.storage, &sender_address, &user)?;

    Ok(response.add_attribute("prize", prize))
}

pub fn try_claim(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    check_pause_state(deps.storage)?;
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    let mut app_info = APP_INFO.load(deps.storage)?;
    let mut user = USERS
        .load(deps.storage, &sender_address)
        .unwrap_or_default();

    // check rewards
    if user.unclaimed.is_zero() {
        Err(ContractError::ZeroRewardsAmount)?;
    }

    // check app balance
    if user.unclaimed > app_info.balance {
        Err(ContractError::NotEnoughLiquidity)?;
    }

    let msg = get_transfer_msg(
        &sender_address,
        user.unclaimed,
        &Token::new_native(&config.denom),
    )?;

    app_info.balance -= user.unclaimed;
    app_info.user_unclaimed -= user.unclaimed;
    user.unclaimed = Uint128::zero();

    APP_INFO.save(deps.storage, &app_info)?;
    USERS.save(deps.storage, &sender_address, &user)?;

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "try_claim"))
}

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
    amount: Option<Uint128>,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let (sender_address, ..) = check_funds(deps.as_ref(), &info, FundsType::Empty)?;
    let config = CONFIG.load(deps.storage)?;
    let mut amount_to_send = Uint128::zero();

    check_authorization(
        &sender_address,
        &config.admin,
        &config.worker,
        AuthType::Admin,
    )?;

    APP_INFO.update(deps.storage, |mut x| -> StdResult<_> {
        amount_to_send =
            amount.unwrap_or(calc_available_to_withdraw(x.deposited, x.revenue.current));

        if amount_to_send.is_zero() {
            Err(ContractError::ZeroAmount)?;
        }

        // don't allow to withdraw funds required to pay unclaimed rewards
        if calc_available_to_withdraw(x.deposited, x.revenue.current) < amount_to_send {
            Err(ContractError::NotEnoughLiquidity)?;
        }
        x.balance -= amount_to_send;

        // withdraw deposited first
        if x.deposited >= amount_to_send {
            x.deposited -= amount_to_send;
        } else {
            let diff = Int256::from(amount_to_send - x.deposited);
            x.deposited = Uint128::zero();

            // then withdraw current revenue
            x.revenue.current -= diff;
        }

        Ok(x)
    })?;

    let recipient = recipient
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(sender_address);
    let msg = get_transfer_msg(
        &recipient,
        amount_to_send,
        &Token::new_native(&config.denom),
    )?;

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
        if x.min > x.max {
            Err(ContractError::ImproperMinBet)?;
        }

        if x.max.is_zero() {
            Err(ContractError::ZeroMaxBet)?;
        }

        config.bet = x;
        is_config_updated = true;
    }

    if let Some(x) = platform_fee {
        if x > Decimal::one() {
            Err(ContractError::FeeIsOutOfRange)?;
        }

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
