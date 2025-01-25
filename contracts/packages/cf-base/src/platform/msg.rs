use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};

use super::types::{Range, Side, UserInfo};

#[cw_serde]
pub struct MigrateMsg {
    pub version: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub worker: Option<String>,
    pub bet: Option<Range>,
    pub platform_fee: Option<Decimal>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // users
    Flip {
        side: Side,
    },

    Claim {},

    // new_admin
    AcceptAdminRole {},

    // admin, worker
    Deposit {},

    Withdraw {
        amount: Uint128,
        recipient: Option<String>,
    },

    UpdateConfig {
        admin: Option<String>,
        worker: Option<String>,
        bet: Option<Range>,
        platform_fee: Option<Decimal>,
    },

    Pause {},

    Unpause {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(super::types::Config)]
    Config {},

    #[returns(super::types::AppInfo)]
    AppInfo {},

    #[returns(Uint128)]
    AvailableToWithdraw {},

    #[returns(super::types::UserInfo)]
    User { address: String },

    #[returns(Vec<UserListRespItem>)]
    UserList {
        amount: u32,
        start_after: Option<String>,
    },
}

#[cw_serde]
pub struct UserListRespItem {
    pub address: Addr,
    pub info: UserInfo,
}
