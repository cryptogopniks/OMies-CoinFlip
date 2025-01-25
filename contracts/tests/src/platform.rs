use cosmwasm_std::{Int256, StdResult};
use cw_multi_test::Executor;

use rand::Rng;

use cf_base::platform::{
    msg::MigrateMsg,
    types::{AppInfo, Range, Side},
};
use speculoos::assert_that;

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
fn ns_matters_no_delay() -> StdResult<()> {
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

    let AppInfo { user_stats, .. } = p.platform_query_app_info()?;
    assert_that(&user_stats.wins.value.u128()).is_equal_to(896_000);

    Ok(())
}

#[test]
fn ns_matters_with_delay() -> StdResult<()> {
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

    // even 1 ns will affect on result
    p.app.update_block(|x| {
        x.time = x.time.plus_nanos(1);
    });

    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    let AppInfo { user_stats, .. } = p.platform_query_app_info()?;
    assert_that(&user_stats.wins.value.u128()).is_equal_to(902_000);

    Ok(())
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

    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(104_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(104_000);

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&user_stats.wins.count).is_equal_to(448);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(896_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.104");
    assert_that(&balance.u128()).is_equal_to(104_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(104_000));

    Ok(())
}

// let num = rand::thread_rng().gen_range(0..100);

// TODO
// guards
// big numbers
// balance manipulations
// +ns matters
// claim/unclaimed
// +const_period_const_amount_const_side
// var_period_const_amount_const_side
// const_period_var_amount_const_side
// const_period_const_amount_var_side
// var_period_var_amount_const_side
// const_period_var_amount_var_side
// var_period_var_amount_var_side
// multiple_users
