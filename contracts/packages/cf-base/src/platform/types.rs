use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Int256, StdResult, Uint128};

use crate::{
    converters::{str_to_dec, u128_to_dec},
    error::ContractError,
};

#[cw_serde]
pub enum Side {
    Head,
    Tail,
}

impl Side {
    pub fn is_winner(&self, random_weight: Decimal, platform_fee: Decimal) -> bool {
        let offset = platform_fee / u128_to_dec(2_u128);
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

impl StatsItem {
    pub fn increase(&mut self, value: Uint128) {
        self.count += 1;
        self.value += value;
    }
}

fn get_user_gain(bets: &StatsItem, wins: &StatsItem) -> Decimal {
    if bets.value.is_zero() {
        return Decimal::one();
    }

    u128_to_dec(wins.value) / u128_to_dec(bets.value)
}

#[derive(Default)]
#[cw_serde]
pub struct UserInfo {
    pub stats: Stats,
    /// user_gain = user_wins / user_bets
    pub gain: Decimal,
    pub unclaimed: Uint128,
    pub last_flip_date: u64,
}

impl UserInfo {
    pub fn update_gain(&mut self) {
        self.gain = get_user_gain(&self.stats.bets, &self.stats.wins);
    }
}

#[derive(Default)]
#[cw_serde]
pub struct Revenue {
    /// to track how much revenue was generated since the beginning
    pub total: Int256,
    /// to track how much revenue wasn't withdrawn at current moment, decreased on withdraw
    pub current: Int256,
}

#[derive(Default)]
#[cw_serde]
pub struct AppInfo {
    /// total user stats
    pub user_stats: Stats,
    /// total user unclaimed
    pub user_unclaimed: Uint128,

    /// app_gain = 2 - user_wins / user_bets
    pub gain: Decimal,
    /// increased on deposit
    /// decreased on withdraw
    pub deposited: Uint128,
    /// balance = revenue_current + deposited + user_unclaimed
    /// increased on deposit, flip-lose
    /// decreased on withdraw, flip-win (with auto claim), claim
    pub balance: Uint128,
    /// revenue_total ≈ platform_fee * total_bets
    pub revenue: Revenue,
}

impl AppInfo {
    pub fn update_gain(&mut self) {
        self.gain =
            u128_to_dec(2_u128) - get_user_gain(&self.user_stats.bets, &self.user_stats.wins);
    }
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
