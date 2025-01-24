use std::fmt::Debug;

use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, BlockInfo, Coin, Empty, StdResult, Timestamp, Uint128,
};
use cw_multi_test::{
    App, AppBuilder, AppResponse, BankSudo, Executor, MockApiBech32, SudoMsg, WasmKeeper,
};

use serde::Serialize;
use strum::IntoEnumIterator;

use cf_base::{
    assets::{Currency, Funds, Token},
    error::parse_err,
};

use crate::helpers::suite::{
    codes::WithCodes,
    types::{
        CustomApp, GetDecimals, ProjectAccount, ProjectAsset, ProjectCoin, ProjectNft,
        ProjectToken, DEFAULT_DECIMALS,
    },
};

pub struct Project {
    pub app: CustomApp,
    contract_counter: u16,

    // package code id
    cw20_base_code_id: u64,
    cw721_base_code_id: u64,

    // contract code id
    platform_code_id: u64,

    // package address
    gopniks_address: Addr,
    pinjeons_address: Addr,

    // contract address
    platform_address: Addr,
    //
    // other
}

impl Project {
    pub fn create_project_with_balances() -> Self {
        Self {
            app: Self::create_app_with_balances(),
            contract_counter: 0,

            cw20_base_code_id: 0,
            cw721_base_code_id: 0,

            platform_code_id: 0,

            gopniks_address: Addr::unchecked(""),
            pinjeons_address: Addr::unchecked(""),

            platform_address: Addr::unchecked(""),
        }
    }

    pub fn new() -> Self {
        // create app and distribute coins to accounts
        let mut project = Self::create_project_with_balances();

        // register contracts code
        // packages
        let cw20_base_code_id = project.store_cw20_base_code();
        // let cw721_base_code_id = project.store_cw721_base_code();

        // contracts
        let platform_code_id = project.store_platform_code();

        // instantiate packages

        // DON'T CHANGE TOKEN INIT ORDER AS ITS ADDRESSES ARE HARDCODED IN ProjectToken ENUM
        for project_token in ProjectToken::iter() {
            project.instantiate_cw20_base_token(cw20_base_code_id, project_token);
        }

        // for _project_nft in ProjectNft::iter() {
        //     project.instantiate_cw721_base_token(cw721_base_code_id);
        // }

        // // mint NFTs
        // let token_id_list: Vec<u128> = vec![1, 2, 3];
        // for collection in ProjectNft::iter() {
        //     for (i, recipient) in [
        //         ProjectAccount::Alice,
        //         ProjectAccount::Bob,
        //         ProjectAccount::John,
        //         ProjectAccount::Kate,
        //         ProjectAccount::Ruby,
        //     ]
        //     .iter()
        //     .enumerate()
        //     {
        //         let nft_list: &Vec<u128> = &token_id_list
        //             .iter()
        //             .map(|x| x + (i as u128) * (token_id_list.len() as u128))
        //             .collect();

        //         project.mint_nft(ProjectAccount::Owner, recipient, collection, nft_list);
        //     }
        // }

        // instantiate contracts

        let platform_address = project.instantiate_platform(platform_code_id, None, None, None);

        project = Self {
            cw20_base_code_id,

            platform_code_id,

            gopniks_address: Addr::unchecked(ProjectNft::Gopniks.to_string()),
            pinjeons_address: Addr::unchecked(ProjectNft::Pinjeons.to_string()),

            platform_address,

            ..project
        };

        // prepare contracts

        project
    }

    // code id getters
    pub fn get_cw20_base_code_id(&self) -> u64 {
        self.cw20_base_code_id
    }

    pub fn get_cw721_base_code_id(&self) -> u64 {
        self.cw721_base_code_id
    }

    pub fn get_platform_code_id(&self) -> u64 {
        self.platform_code_id
    }

    // package address getters
    pub fn get_gopniks_address(&self) -> Addr {
        self.gopniks_address.clone()
    }

    pub fn get_pinjeons_address(&self) -> Addr {
        self.pinjeons_address.clone()
    }

    // contract address getters
    pub fn get_platform_address(&self) -> Addr {
        self.platform_address.clone()
    }

    // utils
    pub fn addr(&self, acc: impl ToString) -> Addr {
        self.app.api().addr_make(&acc.to_string())
    }

    pub fn increase_contract_counter(&mut self, step: u16) {
        self.contract_counter += step;
    }

    pub fn get_last_contract_address(&self) -> String {
        format!("contract{}", self.contract_counter)
    }

    pub fn get_block_time(&self) -> u64 {
        self.app.block_info().time.seconds()
    }

    pub fn reset_time(&mut self) {
        self.app.update_block(|block| {
            block.time = Timestamp::default().plus_seconds(1_000);
            block.height = 200;
        });
    }

    pub fn wait(&mut self, delay_s: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_seconds(delay_s);
            block.height += delay_s / 5;
        });
    }

    pub fn set_chain_id(&mut self, chain_id: &str) {
        self.app.update_block(|block| {
            block.chain_id = chain_id.to_string();
        });
    }

    pub fn mint_native(&mut self, recipient: impl ToString, amount: u128, asset: ProjectCoin) {
        self.app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: recipient.to_string(),
                amount: coins(amount, asset.to_string()),
            }))
            .unwrap();
    }

    pub fn increase_allowances(
        &mut self,
        owner: ProjectAccount,
        spender: impl ToString,
        assets: &[(impl Into<Uint128> + Clone, ProjectToken)],
    ) {
        for (asset_amount, token) in assets {
            self.app
                .execute_contract(
                    owner.into(),
                    token.to_owned().into(),
                    &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                        spender: spender.to_string(),
                        amount: asset_amount.to_owned().into(),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
        }
    }

    pub fn increase_allowances_nft(
        &mut self,
        owner: ProjectAccount,
        spender: impl ToString,
        collection: &Addr,
    ) {
        self.app
            .execute_contract(
                owner.into(),
                collection.to_owned(),
                &cw721_base::ExecuteMsg::ApproveAll::<Empty, Empty> {
                    operator: spender.to_string(),
                    expires: None,
                },
                &[],
            )
            .unwrap();
    }

    pub fn mint_nft(
        &mut self,
        owner: ProjectAccount,
        recipient: impl ToString,
        collection: ProjectNft,
        token_id_list: &Vec<impl ToString>,
    ) {
        for token_id in token_id_list {
            let msg = &cw721_base::msg::ExecuteMsg::Mint::<Empty, Empty> {
                token_id: token_id.to_string(),
                owner: recipient.to_string(),
                token_uri: Some(format!("https://www.{:#?}.com", collection)),
                extension: Empty::default(),
            };

            self.app
                .execute_contract(owner.into(), collection.into(), msg, &[])
                .unwrap();
        }
    }

    pub fn transfer_nft(
        &mut self,
        owner: ProjectAccount,
        recipient: impl ToString,
        collection: impl Into<Addr>,
        token_id: impl ToString,
    ) {
        let msg = &cw721_base::msg::ExecuteMsg::TransferNft::<Empty, Empty> {
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        };

        self.app
            .execute_contract(owner.into(), collection.into(), msg, &[])
            .unwrap();
    }

    pub fn query_nft(&self, owner: impl ToString, collection: impl ToString) -> Vec<String> {
        self.app
            .wrap()
            .query_wasm_smart::<cw721::TokensResponse>(
                collection.to_string(),
                &cw721::Cw721QueryMsg::Tokens {
                    owner: owner.to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
            .tokens
    }

    pub fn query_all_nft(&self, owner: impl ToString) -> Vec<(ProjectNft, Vec<String>)> {
        ProjectNft::iter()
            .map(|collection| {
                let cw721::TokensResponse { tokens } = self
                    .app
                    .wrap()
                    .query_wasm_smart(
                        collection.to_string(),
                        &cw721::Cw721QueryMsg::Tokens {
                            owner: owner.to_string(),
                            start_after: None,
                            limit: None,
                        },
                    )
                    .unwrap();

                (collection, tokens)
            })
            .collect()
    }

    pub fn query_balance(
        &self,
        account: impl ToString,
        token: &(impl Into<Token> + Clone),
    ) -> StdResult<u128> {
        let token: Token = token.to_owned().into();

        match token {
            Token::Native { denom } => Ok(self
                .app
                .wrap()
                .query_balance(account.to_string(), denom)?
                .amount
                .u128()),
            Token::Cw20 { address } => {
                let cw20::BalanceResponse { balance } = self.app.wrap().query_wasm_smart(
                    address.to_string(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: account.to_string(),
                    },
                )?;

                Ok(balance.u128())
            }
        }
    }

    pub fn query_all_balances(&self, account: impl ToString) -> StdResult<Vec<Funds<Token>>> {
        let mut funds_list: Vec<Funds<Token>> = vec![];

        for token_native in ProjectCoin::iter() {
            let Coin { denom, amount } = self
                .app
                .wrap()
                .query_balance(account.to_string(), token_native.to_string())?;

            if !amount.is_zero() {
                funds_list.push(Funds::new(
                    amount,
                    &Currency::new(&Token::new_native(&denom), DEFAULT_DECIMALS),
                ));
            }
        }

        for token_cw20 in ProjectToken::iter() {
            let cw20::BalanceResponse { balance } = self.app.wrap().query_wasm_smart(
                token_cw20.to_string(),
                &cw20::Cw20QueryMsg::Balance {
                    address: account.to_string(),
                },
            )?;

            if !balance.is_zero() {
                funds_list.push(Funds::new(
                    balance,
                    &Currency::new(
                        &Token::new_cw20(&token_cw20.into()),
                        token_cw20.get_decimals(),
                    ),
                ));
            }
        }

        Ok(funds_list)
    }

    pub fn instantiate_contract(
        &mut self,
        code_id: u64,
        label: &str,
        init_msg: &impl Serialize,
    ) -> Addr {
        self.increase_contract_counter(1);

        self.app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ProjectAccount::Admin.to_string()),
                init_msg,
                &[],
                label,
                Some(ProjectAccount::Admin.to_string()),
            )
            .unwrap()
    }

    pub fn create_app_with_balances() -> CustomApp {
        let block_info = App::default().block_info();

        AppBuilder::new_custom()
            .with_api(MockApiBech32::new("wasm"))
            .with_wasm(WasmKeeper::new())
            .with_block(BlockInfo {
                height: block_info.height,
                time: block_info.time,
                chain_id: "wasm-1".to_string(),
            })
            .build(|router, _api, storage| {
                for project_account in ProjectAccount::iter() {
                    let funds: Vec<Coin> = ProjectCoin::iter()
                        .map(|project_coin| {
                            let amount = project_account.get_initial_funds_amount()
                                * 10u128.pow(project_coin.get_decimals() as u32);

                            coin(amount, project_coin.to_string())
                        })
                        .collect();

                    router
                        .bank
                        .init_balance(storage, &project_account.into(), funds)
                        .unwrap();
                }
            })
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

pub fn assert_error<S: std::fmt::Debug + ToString>(
    subject: &S,
    err: impl ToString + Sized + Debug,
) {
    let expected_error_name = &format!("{:#?}", err);
    let expected_error_text = &err.to_string();

    speculoos::assert_that(subject).matches(|x| {
        let error = format!("{:#?}", x);

        error.contains(expected_error_name) || error.contains(expected_error_text)
    });
}

pub fn add_funds_to_exec_msg<T: Serialize + std::fmt::Debug>(
    project: &mut Project,
    sender: ProjectAccount,
    contract_address: &Addr,
    msg: &T,
    amount: impl Into<Uint128>,
    asset: impl Into<ProjectAsset>,
) -> StdResult<AppResponse> {
    let asset: ProjectAsset = asset.into();

    match asset {
        ProjectAsset::Coin(denom) => project
            .app
            .execute_contract(
                sender.into(),
                contract_address.to_owned(),
                msg,
                &[coin(
                    Into::<Uint128>::into(amount).u128(),
                    denom.to_string(),
                )],
            )
            .map_err(parse_err),
        ProjectAsset::Token(address) => {
            let wasm_msg = cw20::Cw20ExecuteMsg::Send {
                contract: contract_address.to_string(),
                amount: Into::<Uint128>::into(amount),
                msg: to_json_binary(msg).unwrap(),
            };

            project
                .app
                .execute_contract(sender.into(), address.into(), &wasm_msg, &[])
                .map_err(parse_err)
        }
    }
}

pub fn add_token_to_exec_msg<T: Serialize + std::fmt::Debug>(
    project: &mut Project,
    sender: ProjectAccount,
    contract_address: &Addr,
    msg: &T,
    amount: impl Into<Uint128>,
    asset: &Token,
) -> StdResult<AppResponse> {
    match asset {
        Token::Native { denom } => project
            .app
            .execute_contract(
                sender.into(),
                contract_address.to_owned(),
                msg,
                &[coin(
                    Into::<Uint128>::into(amount).u128(),
                    denom.to_string(),
                )],
            )
            .map_err(parse_err),
        Token::Cw20 { address } => {
            let wasm_msg = cw20::Cw20ExecuteMsg::Send {
                contract: contract_address.to_string(),
                amount: Into::<Uint128>::into(amount),
                msg: to_json_binary(msg).unwrap(),
            };

            project
                .app
                .execute_contract(sender.into(), address.to_owned(), &wasm_msg, &[])
                .map_err(parse_err)
        }
    }
}

pub fn to_string_vec(str_vec: &[&str]) -> Vec<String> {
    str_vec.iter().map(|x| x.to_string()).collect()
}
