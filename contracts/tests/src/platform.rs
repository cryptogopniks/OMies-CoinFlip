use cosmwasm_std::StdResult;
use cw_multi_test::Executor;

use rand::Rng;

use cf_base::platform::{
    msg::MigrateMsg,
    types::{Range, Side},
};

use crate::helpers::{
    platform::PlatformExtension,
    suite::{
        core::Project,
        types::{ProjectAccount, ProjectCoin},
    },
};

#[test]
fn migrate_default() {
    let mut p = Project::new();

    p.app
        .migrate_contract(
            ProjectAccount::Admin.into(),
            p.get_platform_address(),
            &MigrateMsg {
                version: "1.0.0".to_string(),
            },
            p.get_platform_code_id(),
        )
        .unwrap();
}

#[test]
fn const_period_const_amount_const_side() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY: u64 = 3;
    const SIDE: Side = Side::Head;
    const AMOUNT: u128 = 1_000;

    let mut p = Project::new();

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT)),
        None,
    )?;

    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    p.platform_try_withdraw(ProjectAccount::Admin, 20_000, None)?;
    p.platform_try_deposit(ProjectAccount::Admin, 6_000, ProjectCoin::Om)?;

    let available_to_withdraw = p.platform_query_available_to_withdraw()?;
    let app_info = p.platform_query_app_info()?;

    println!(
        "available_to_withdraw {:#?}\n",
        available_to_withdraw.u128()
    );
    println!("app_info {:#?}\n", app_info);

    // let num = rand::thread_rng().gen_range(0..100);

    Ok(())
}
