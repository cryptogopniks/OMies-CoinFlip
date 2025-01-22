use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, StdResult, Uint128};

use crate::{converters::str_to_dec, error::ContractError};

#[cw_serde]
pub enum Side {
    Head,
    Tail,
}

impl Side {
    pub fn is_winner(&self, random_weight: Decimal, platform_fee: Decimal) -> bool {
        let offset = platform_fee / str_to_dec("2");
        let lower_threshold = str_to_dec("0.5") - offset;
        let higher_threshold = str_to_dec("0.5") + offset;

        match self {
            Self::Head => random_weight <= lower_threshold,
            Self::Tail => random_weight >= higher_threshold,
        }
    }
}

#[derive(Default)]
#[cw_serde]
pub struct Stats {
    pub bets: StatsItem,
    pub wins: StatsItem,
}

#[derive(Default)]
#[cw_serde]
pub struct StatsItem {
    pub count: u32,
    pub value: Uint128,
}

#[derive(Default)]
#[cw_serde]
pub struct UserInfo {
    pub stats: Stats,
    /// gain = wins / bets
    pub gain: Decimal,
    pub unclaimed: Uint128,
    pub last_flip_date: u64,
}

#[derive(Default)]
#[cw_serde]
pub struct AppInfo {
    /// total user stats
    pub user_stats: Stats,
    /// gain = wins / bets
    pub user_gain: Decimal,
    /// total user unclaimed
    pub user_unclaimed: Uint128,

    /// increased on deposit
    /// decreased on withdraw
    pub deposited: Uint128,
    /// increased on deposit, flip-lose
    /// decreased on withdraw, flip-win (with auto claim), claim
    pub balance: Uint128,
    /// revenue = balance - deposited - user_unclaimed
    /// revenue ≈ platform_fee * total_bets
    pub revenue: Uint128,
}

#[cw_serde]
pub struct Range {
    pub min: Uint128,
    pub max: Uint128,
}

impl Range {
    pub fn new<T>(min: T, max: T) -> Self
    where
        Uint128: From<T>,
    {
        Self {
            min: Uint128::from(min),
            max: Uint128::from(max),
        }
    }

    pub fn validate(&self, bet: Uint128) -> StdResult<()> {
        if bet < self.min || bet > self.max {
            Err(ContractError::BetIsOutOfRange)?;
        }

        Ok(())
    }
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub worker: Option<Addr>,
    pub bet: Range,
    pub denom: String,
    pub platform_fee: Decimal,
}

#[cw_serde]
pub struct TransferAdminState {
    pub new_admin: Addr,
    pub deadline: u64,
}
