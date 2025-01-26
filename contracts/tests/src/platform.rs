use cosmwasm_std::{Int256, StdResult};
use cw_multi_test::Executor;

use rand::{rngs::StdRng, Rng, SeedableRng};

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

fn get_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

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

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(104_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(104_000);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(1_000_000);
    assert_that(&user_stats.wins.count).is_equal_to(448);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(896_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.104");
    assert_that(&balance.u128()).is_equal_to(104_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(104_000));

    Ok(())
}

#[test]
fn var_period_const_amount_const_side() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY_MAX: u64 = 15;
    const SIDE: Side = Side::Head;
    const AMOUNT: u128 = 1_000;

    let mut p = Project::new();
    let mut rng = get_rng(42);

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
        p.wait(rng.gen_range(3..=DELAY_MAX));
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(168_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(168_000);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(1_000_000);
    assert_that(&user_stats.wins.count).is_equal_to(416);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(832_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.168");
    assert_that(&balance.u128()).is_equal_to(168_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(168_000));

    Ok(())
}

#[test]
fn const_period_var_amount_const_side() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY: u64 = 3;
    const SIDE: Side = Side::Head;
    const AMOUNT_MIN: u128 = 1_000;
    const AMOUNT_MAX: u128 = 5_000;

    let mut p = Project::new();
    let mut rng = get_rng(42);

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT_MAX)),
        None,
    )?;

    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        p.platform_try_flip(
            ProjectAccount::Alice,
            SIDE,
            rng.gen_range(AMOUNT_MIN..=AMOUNT_MAX),
            ProjectCoin::Om,
        )?;
        p.wait(DELAY);
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(351_770);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(351_770);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(3_035_156);
    assert_that(&user_stats.wins.count).is_equal_to(448);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(2_683_386);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.115898490884817783");
    assert_that(&balance.u128()).is_equal_to(351_770);
    assert_that(&revenue.total).is_equal_to(Int256::from(351_770));

    Ok(())
}

#[test]
fn const_period_const_amount_var_side() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY: u64 = 3;
    const AMOUNT: u128 = 1_000;

    let mut p = Project::new();
    let mut rng = get_rng(42);

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
        let side = if rng.gen_range(0..=1) == 0 {
            Side::Head
        } else {
            Side::Tail
        };
        p.platform_try_flip(ProjectAccount::Alice, side, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(92_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(92_000);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(1_000_000);
    assert_that(&user_stats.wins.count).is_equal_to(454);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(908_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.092");
    assert_that(&balance.u128()).is_equal_to(92_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(92_000));

    Ok(())
}

#[test]
fn var_period_var_amount_const_side() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY_MAX: u64 = 15;
    const SIDE: Side = Side::Head;
    const AMOUNT_MIN: u128 = 1_000;
    const AMOUNT_MAX: u128 = 5_000;

    let mut p = Project::new();
    let mut rng_1 = get_rng(42);
    let mut rng_2 = get_rng(43);

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT_MAX)),
        None,
    )?;

    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        p.platform_try_flip(
            ProjectAccount::Alice,
            SIDE,
            rng_2.gen_range(AMOUNT_MIN..=AMOUNT_MAX),
            ProjectCoin::Om,
        )?;
        p.wait(rng_1.gen_range(3..=DELAY_MAX));
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(500_281);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(500_281);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(3_028_729);
    assert_that(&user_stats.wins.count).is_equal_to(416);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(2_528_448);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.165178528683153891");
    assert_that(&balance.u128()).is_equal_to(500_281);
    assert_that(&revenue.total).is_equal_to(Int256::from(500_281));

    Ok(())
}

#[test]
fn const_period_var_amount_var_side() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY: u64 = 3;
    const AMOUNT_MIN: u128 = 1_000;
    const AMOUNT_MAX: u128 = 5_000;

    let mut p = Project::new();
    let mut rng_1 = get_rng(42);
    let mut rng_2 = get_rng(43);

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT_MAX)),
        None,
    )?;

    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        let side = if rng_2.gen_range(0..=1) == 0 {
            Side::Head
        } else {
            Side::Tail
        };
        p.platform_try_flip(
            ProjectAccount::Alice,
            side,
            rng_1.gen_range(AMOUNT_MIN..=AMOUNT_MAX),
            ProjectCoin::Om,
        )?;
        p.wait(DELAY);
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(249_596);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(249_596);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(3_035_156);
    assert_that(&user_stats.wins.count).is_equal_to(444);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(2_785_560);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.082234982320513345");
    assert_that(&balance.u128()).is_equal_to(249_596);
    assert_that(&revenue.total).is_equal_to(Int256::from(249_596));

    Ok(())
}

#[test]
fn var_period_var_amount_var_side() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY_MAX: u64 = 15;
    const AMOUNT_MIN: u128 = 1_000;
    const AMOUNT_MAX: u128 = 5_000;

    let mut p = Project::new();
    let mut rng_1 = get_rng(42);
    let mut rng_2 = get_rng(43);
    let mut rng_3 = get_rng(44);

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT_MAX)),
        None,
    )?;

    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        let side = if rng_3.gen_range(0..=1) == 0 {
            Side::Head
        } else {
            Side::Tail
        };
        p.platform_try_flip(
            ProjectAccount::Alice,
            side,
            rng_2.gen_range(AMOUNT_MIN..=AMOUNT_MAX),
            ProjectCoin::Om,
        )?;
        p.wait(rng_1.gen_range(3..=DELAY_MAX));
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(373_471);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(373_471);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(3_028_729);
    assert_that(&user_stats.wins.count).is_equal_to(449);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(2_655_258);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.12330948064353067");
    assert_that(&balance.u128()).is_equal_to(373_471);
    assert_that(&revenue.total).is_equal_to(Int256::from(373_471));

    Ok(())
}

// TODO
// guards
// big numbers
// balance manipulations
// +ns matters
// claim/unclaimed
// +const_period_const_amount_const_side
// +var_period_const_amount_const_side
// +const_period_var_amount_const_side
// +const_period_const_amount_var_side
// +var_period_var_amount_const_side
// +const_period_var_amount_var_side
// +var_period_var_amount_var_side
// multiple_users
