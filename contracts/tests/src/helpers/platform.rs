use cosmwasm_std::{StdResult, Uint128};
use cw_multi_test::{AppResponse, Executor};

use cf_base::{
    converters::str_to_dec,
    error::parse_err,
    platform::{
        msg::{ExecuteMsg, QueryMsg, UserListRespItem},
        types::{AppInfo, Config, Range, Side, UserInfo},
    },
};

use crate::helpers::suite::{
    core::{add_funds_to_exec_msg, Project},
    types::ProjectAccount,
};

use super::suite::types::ProjectAsset;

pub trait PlatformExtension {
    fn platform_try_flip(
        &mut self,
        sender: ProjectAccount,
        side: Side,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn platform_try_claim(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn platform_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn platform_try_deposit(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse>;

    fn platform_try_withdraw(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        recipient: Option<ProjectAccount>,
    ) -> StdResult<AppResponse>;

    fn platform_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: Option<ProjectAccount>,
        worker: Option<ProjectAccount>,
        bet: Option<Range>,
        platform_fee: Option<&str>,
    ) -> StdResult<AppResponse>;

    fn lending_platform_try_pause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn lending_platform_try_unpause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse>;

    fn platform_query_config(&self) -> StdResult<Config>;

    fn platform_query_app_info(&self) -> StdResult<AppInfo>;

    fn platform_query_user(&self, address: impl ToString) -> StdResult<UserInfo>;

    fn platform_query_user_list(
        &self,
        amount: u32,
        start_after: Option<&str>,
    ) -> StdResult<Vec<UserListRespItem>>;
}

impl PlatformExtension for Project {
    #[track_caller]
    fn platform_try_flip(
        &mut self,
        sender: ProjectAccount,
        side: Side,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        add_funds_to_exec_msg(
            self,
            sender,
            &self.get_platform_address(),
            &ExecuteMsg::Flip { side },
            amount,
            asset,
        )
    }

    #[track_caller]
    fn platform_try_claim(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_platform_address(),
                &ExecuteMsg::Claim {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn platform_try_accept_admin_role(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_platform_address(),
                &ExecuteMsg::AcceptAdminRole {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn platform_try_deposit(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        asset: impl Into<ProjectAsset>,
    ) -> StdResult<AppResponse> {
        add_funds_to_exec_msg(
            self,
            sender,
            &self.get_platform_address(),
            &ExecuteMsg::Deposit {},
            amount,
            asset,
        )
    }

    #[track_caller]
    fn platform_try_withdraw(
        &mut self,
        sender: ProjectAccount,
        amount: u128,
        recipient: Option<ProjectAccount>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_platform_address(),
                &ExecuteMsg::Withdraw {
                    amount: Uint128::new(amount),
                    recipient: recipient.map(|x| x.to_string()),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn platform_try_update_config(
        &mut self,
        sender: ProjectAccount,
        admin: Option<ProjectAccount>,
        worker: Option<ProjectAccount>,
        bet: Option<Range>,
        platform_fee: Option<&str>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_platform_address(),
                &ExecuteMsg::UpdateConfig {
                    admin: admin.map(|x| x.to_string()),
                    worker: worker.map(|x| x.to_string()),
                    bet,
                    platform_fee: platform_fee.map(str_to_dec),
                },
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_pause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_platform_address(),
                &ExecuteMsg::Pause {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn lending_platform_try_unpause(&mut self, sender: ProjectAccount) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                sender.into(),
                self.get_platform_address(),
                &ExecuteMsg::Unpause {},
                &[],
            )
            .map_err(parse_err)
    }

    #[track_caller]
    fn platform_query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_platform_address(), &QueryMsg::Config {})
    }

    #[track_caller]
    fn platform_query_app_info(&self) -> StdResult<AppInfo> {
        self.app
            .wrap()
            .query_wasm_smart(self.get_platform_address(), &QueryMsg::AppInfo {})
    }

    #[track_caller]
    fn platform_query_user(&self, address: impl ToString) -> StdResult<UserInfo> {
        self.app.wrap().query_wasm_smart(
            self.get_platform_address(),
            &QueryMsg::User {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    fn platform_query_user_list(
        &self,
        amount: u32,
        start_after: Option<&str>,
    ) -> StdResult<Vec<UserListRespItem>> {
        self.app.wrap().query_wasm_smart(
            self.get_platform_address(),
            &QueryMsg::UserList {
                amount,
                start_after: start_after.map(|x| x.to_string()),
            },
        )
    }
}
