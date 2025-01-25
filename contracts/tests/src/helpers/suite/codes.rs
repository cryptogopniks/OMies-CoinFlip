use cosmwasm_std::{Addr, StdResult, Uint128};
use cw_multi_test::{AppResponse, ContractWrapper, Executor};

use serde::Serialize;
use strum::IntoEnumIterator;

use cf_base::{converters::str_to_dec, error::parse_err, platform::types::Range};

use crate::helpers::suite::{
    core::Project,
    types::{GetDecimals, ProjectAccount, ProjectToken},
};

pub trait WithCodes {
    // store packages
    fn store_cw20_base_code(&mut self) -> u64;
    // fn store_cw721_base_code(&mut self) -> u64;

    // store contracts
    fn store_platform_code(&mut self) -> u64;

    // instantiate packages
    fn instantiate_cw20_base_token(&mut self, code_id: u64, project_token: ProjectToken) -> Addr;
    fn instantiate_cw721_base_token(&mut self, code_id: u64) -> Addr;

    // instantiate contracts
    fn instantiate_platform(
        &mut self,
        platform_code_id: u64,
        worker: Option<ProjectAccount>,
        bet: Option<Range>,
        platform_fee: Option<&str>,
    ) -> Addr;

    fn migrate_contract(
        &mut self,
        sender: ProjectAccount,
        contract_address: Addr,
        contract_new_code_id: u64,
        migrate_msg: impl Serialize,
    ) -> StdResult<AppResponse>;
}

impl WithCodes for Project {
    // store packages
    fn store_cw20_base_code(&mut self) -> u64 {
        self.app.store_code(Box::new(ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        )))
    }

    // fn store_cw721_base_code(&mut self) -> u64 {
    //     self.app.store_code(Box::new(ContractWrapper::new(
    //         cw721_base::entry::execute,
    //         cw721_base::entry::instantiate,
    //         cw721_base::entry::query,
    //     )))
    // }

    // store contracts
    fn store_platform_code(&mut self) -> u64 {
        self.app.store_code(Box::new(
            ContractWrapper::new(
                platform::contract::execute,
                platform::contract::instantiate,
                platform::contract::query,
            )
            .with_reply(platform::contract::reply)
            .with_migrate(platform::contract::migrate),
        ))
    }

    // instantiate packages
    fn instantiate_cw20_base_token(&mut self, code_id: u64, project_token: ProjectToken) -> Addr {
        let symbol = "TOKEN".to_string();

        let initial_balances: Vec<cw20::Cw20Coin> = ProjectAccount::iter()
            .map(|project_account| {
                let amount = project_account.get_initial_funds_amount()
                    * 10u128.pow(project_token.get_decimals() as u32);

                cw20::Cw20Coin {
                    address: project_account.to_string(),
                    amount: Uint128::from(amount),
                }
            })
            .collect();

        self.instantiate_contract(
            code_id,
            "token",
            &cw20_base::msg::InstantiateMsg {
                name: format!("cw20-base token {}", symbol),
                symbol,
                decimals: project_token.get_decimals(),
                initial_balances,
                mint: None,
                marketing: None,
            },
        )
    }

    fn instantiate_cw721_base_token(&mut self, code_id: u64) -> Addr {
        let symbol = "NFT XYZ".to_string(); // max 10 tokens

        self.instantiate_contract(
            code_id,
            "nft xyz",
            &cw721_base::msg::InstantiateMsg {
                name: format!("cw721-base token {}", symbol),
                symbol,
                minter: ProjectAccount::Owner.to_string(),
            },
        )
    }

    // instantiate contracts
    fn instantiate_platform(
        &mut self,
        platform_code_id: u64,
        worker: Option<ProjectAccount>,
        bet: Option<Range>,
        platform_fee: Option<&str>,
    ) -> Addr {
        self.instantiate_contract(
            platform_code_id,
            "platform",
            &cf_base::platform::msg::InstantiateMsg {
                worker: worker.map(|x| x.to_string()),
                bet,
                platform_fee: platform_fee.map(str_to_dec),
            },
        )
    }

    fn migrate_contract(
        &mut self,
        sender: ProjectAccount,
        contract_address: Addr,
        contract_new_code_id: u64,
        migrate_msg: impl Serialize,
    ) -> StdResult<AppResponse> {
        self.app
            .migrate_contract(
                sender.into(),
                contract_address,
                &migrate_msg,
                contract_new_code_id,
            )
            .map_err(parse_err)
    }
}
