use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Int256, SignedDecimal, StdResult, Uint128};

use crate::{
    converters::{str_to_dec, str_to_sdec, u128_to_dec},
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

fn get_user_roi(bets: &StatsItem, wins: &StatsItem) -> SignedDecimal {
    if bets.value.is_zero() {
        return SignedDecimal::zero();
    }

    let ratio = u128_to_dec(wins.value) / u128_to_dec(bets.value);
    str_to_sdec(&ratio.to_string()) - SignedDecimal::one()
}

#[derive(Default)]
#[cw_serde]
pub struct UserInfo {
    pub stats: Stats,
    /// user_roi = user_wins / user_bets - 1
    pub roi: SignedDecimal,
    pub unclaimed: Uint128,
    pub last_flip_date: u64,
}

impl UserInfo {
    pub fn update_roi(&mut self) {
        self.roi = get_user_roi(&self.stats.bets, &self.stats.wins);
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

    /// average_fee = 1 - user_wins / user_bets
    pub average_fee: SignedDecimal,
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
    pub fn update_average_fee(&mut self) {
        self.average_fee = -get_user_roi(&self.user_stats.bets, &self.user_stats.wins);
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
