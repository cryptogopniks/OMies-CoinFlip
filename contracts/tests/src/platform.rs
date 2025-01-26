use cosmwasm_std::{Int256, StdResult, Uint128};
use cw_multi_test::Executor;

use rand::{rngs::StdRng, Rng, SeedableRng};

use cf_base::{
    error::ContractError,
    platform::{
        msg::MigrateMsg,
        types::{AppInfo, Range, Side, Stats, StatsItem},
    },
};
use speculoos::assert_that;

use crate::helpers::{
    platform::PlatformExtension,
    suite::{
        core::{assert_error, Project},
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
fn guards() -> StdResult<()> {
    const SIDE: Side = Side::Head;
    const AMOUNT: u128 = 1_000;

    let mut p = Project::new();

    // try update config
    let res = p
        .platform_try_update_config(
            ProjectAccount::Alice,
            None,
            None,
            Some(Range::new(0, AMOUNT)),
            None,
        )
        .unwrap_err();
    assert_error(&res, ContractError::Unauthorized);

    let res = p
        .platform_try_update_config(
            ProjectAccount::Admin,
            None,
            None,
            Some(Range::new(0_u128, 0_u128)),
            None,
        )
        .unwrap_err();
    assert_error(&res, ContractError::ZeroMaxBet);

    let res = p
        .platform_try_update_config(
            ProjectAccount::Admin,
            None,
            None,
            Some(Range::new(2 * AMOUNT, AMOUNT)),
            None,
        )
        .unwrap_err();
    assert_error(&res, ContractError::ImproperMinBet);

    let res = p
        .platform_try_update_config(
            ProjectAccount::Admin,
            None,
            None,
            Some(Range::new(0, AMOUNT)),
            Some("1.5"),
        )
        .unwrap_err();
    assert_error(&res, ContractError::FeeIsOutOfRange);

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT)),
        None,
    )?;

    // flip
    let res = p
        .platform_try_flip(ProjectAccount::Alice, SIDE, 0, ProjectCoin::Om)
        .unwrap_err();
    assert_error(&res, "Cannot transfer empty coins amount");

    let res = p
        .platform_try_flip(ProjectAccount::Alice, SIDE, 2 * AMOUNT, ProjectCoin::Om)
        .unwrap_err();
    assert_error(&res, ContractError::BetIsOutOfRange);

    let res = p
        .platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Usdc)
        .unwrap_err();
    assert_error(&res, ContractError::WrongAssetType);

    p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;

    let res = p
        .platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)
        .unwrap_err();
    assert_error(&res, ContractError::MultipleFlipsPerTx);

    // claim
    let res = p.platform_try_claim(ProjectAccount::Alice).unwrap_err();
    assert_error(&res, ContractError::ZeroRewardsAmount);

    p.platform_try_withdraw(ProjectAccount::Admin, None, None)?;

    p.wait(5);
    p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;

    let res = p.platform_try_claim(ProjectAccount::Alice).unwrap_err();
    assert_error(&res, ContractError::NotEnoughLiquidity);

    // withdraw
    p.platform_try_deposit(ProjectAccount::Admin, 2 * AMOUNT, ProjectCoin::Om)?;

    let res = p
        .platform_try_withdraw(ProjectAccount::Admin, Some(AMOUNT + 1), None)
        .unwrap_err();
    assert_error(&res, ContractError::NotEnoughLiquidity);

    p.platform_try_withdraw(ProjectAccount::Admin, Some(AMOUNT), None)?;

    Ok(())
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

#[test]
fn big_numbers() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY: u64 = 3;
    const SIDE: Side = Side::Head;
    const FEE: &str = "0.87";
    const AMOUNT: u128 = 1_000_000_000_000_000;

    let mut p = Project::new();

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT)),
        Some(FEE),
    )?;

    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    let available_to_withdraw = p.platform_query_available_to_withdraw()?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&available_to_withdraw.u128()).is_equal_to(888_000_000_000_000_000);

    p.platform_try_withdraw(
        ProjectAccount::Admin,
        Some(available_to_withdraw.u128()),
        None,
    )?;

    assert_that(&user_stats.bets.value.u128()).is_equal_to(1_000_000_000_000_000_000);
    assert_that(&user_stats.wins.count).is_equal_to(56);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(112_000_000_000_000_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.888");
    assert_that(&balance.u128()).is_equal_to(888_000_000_000_000_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(888_000_000_000_000_000_i128));

    Ok(())
}

#[test]
fn multiple_users() -> StdResult<()> {
    const ROUNDS: u16 = 1_000;
    const DELAY: u64 = 3;
    const AMOUNT: u128 = 1_000;

    let mut p = Project::new();
    let mut rng_1 = get_rng(42);
    let mut rng_2 = get_rng(43);

    p.platform_try_update_config(
        ProjectAccount::Admin,
        None,
        None,
        Some(Range::new(0, AMOUNT)),
        None,
    )?;

    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let bob_balance_before = p.query_balance(ProjectAccount::Bob, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        let user = if rng_1.gen_range(0..=1) == 0 {
            ProjectAccount::Alice
        } else {
            ProjectAccount::Bob
        };
        let side = if rng_2.gen_range(0..=1) == 0 {
            Side::Head
        } else {
            Side::Tail
        };

        p.platform_try_flip(user, side, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let bob_balance_after = p.query_balance(ProjectAccount::Bob, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let alice_info = p.platform_query_user(ProjectAccount::Alice)?;
    let bob_info = p.platform_query_user(ProjectAccount::Bob)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(61_000);
    assert_that(&(bob_balance_before - bob_balance_after)).is_equal_to(39_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(100_000);

    assert_that(&alice_info.roi.to_string().as_str()).is_equal_to("-0.126293995859213251");
    assert_that(&alice_info.unclaimed.u128()).is_equal_to(0);
    assert_that(&alice_info.stats).is_equal_to(&Stats {
        bets: StatsItem {
            count: 483,
            value: Uint128::new(483_000),
        },
        wins: StatsItem {
            count: 211,
            value: Uint128::new(422_000),
        },
    });
    assert_that(&bob_info.roi.to_string().as_str()).is_equal_to("-0.075435203094777563");
    assert_that(&bob_info.unclaimed.u128()).is_equal_to(0);
    assert_that(&bob_info.stats).is_equal_to(&Stats {
        bets: StatsItem {
            count: 517,
            value: Uint128::new(517_000),
        },
        wins: StatsItem {
            count: 239,
            value: Uint128::new(478_000),
        },
    });

    assert_that(&user_stats.bets.value.u128()).is_equal_to(1_000_000);
    assert_that(&user_stats.wins.count).is_equal_to(450);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(900_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.1");
    assert_that(&balance.u128()).is_equal_to(100_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(100_000));

    Ok(())
}

#[test]
fn win_lose_claim() -> StdResult<()> {
    const ROUNDS: u16 = 2;
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

    p.wait(1);
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

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(0);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(0);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(2_000);
    assert_that(&user_stats.wins.count).is_equal_to(1);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(2_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0");
    assert_that(&balance.u128()).is_equal_to(0);
    assert_that(&revenue.total).is_equal_to(Int256::from(0));

    Ok(())
}

#[test]
fn win_win_lose_deposit_claim() -> StdResult<()> {
    const ROUNDS: u16 = 2;
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

    p.wait(18);
    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    p.wait(1);
    p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;

    p.platform_try_deposit(ProjectAccount::Admin, AMOUNT, ProjectCoin::Om)?;
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

    assert_that(&(alice_balance_after - alice_balance_before)).is_equal_to(1_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(0);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(3_000);
    assert_that(&user_stats.wins.count).is_equal_to(2);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(4_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("-0.333333333333333333");
    assert_that(&balance.u128()).is_equal_to(0);
    assert_that(&revenue.total).is_equal_to(Int256::from(-1_000));

    Ok(())
}

#[test]
fn claiming_is_not_required() -> StdResult<()> {
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

    p.platform_try_deposit(ProjectAccount::Admin, 10 * AMOUNT, ProjectCoin::Om)?;

    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

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
    assert_that(&balance.u128()).is_equal_to(114_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(104_000));

    Ok(())
}

#[test]
fn balance_manipulations() -> StdResult<()> {
    const ROUNDS: u16 = 200;
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

    let admin_balance_before = p.query_balance(ProjectAccount::Admin, &ProjectCoin::Om)?;
    let alice_balance_before = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_before = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    let alice_balance_after = p.query_balance(ProjectAccount::Alice, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        deposited,
        balance,
        revenue,
    } = p.platform_query_app_info()?;

    assert_that(&(alice_balance_before - alice_balance_after)).is_equal_to(36_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(36_000);

    assert_that(&user_stats.bets.value.u128()).is_equal_to(200_000);
    assert_that(&user_stats.wins.count).is_equal_to(87);
    assert_that(&user_stats.wins.value.u128()).is_equal_to(174_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(10_000);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.13");
    assert_that(&deposited.u128()).is_equal_to(0);
    assert_that(&balance.u128()).is_equal_to(36_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(26_000));
    assert_that(&revenue.current).is_equal_to(Int256::from(26_000));

    p.platform_try_withdraw(ProjectAccount::Admin, None, None)?;
    p.platform_try_deposit(ProjectAccount::Admin, AMOUNT, ProjectCoin::Om)?;

    let AppInfo {
        user_unclaimed,
        deposited,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&user_unclaimed.u128()).is_equal_to(10_000);
    assert_that(&deposited.u128()).is_equal_to(1_000);
    assert_that(&balance.u128()).is_equal_to(11_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(26_000));
    assert_that(&revenue.current).is_equal_to(Int256::from(0));

    p.wait(1);
    for _ in 0..ROUNDS {
        p.platform_try_flip(ProjectAccount::Alice, SIDE, AMOUNT, ProjectCoin::Om)?;
        p.wait(DELAY);
    }

    p.platform_try_claim(ProjectAccount::Alice)?;

    let AppInfo {
        user_stats,
        user_unclaimed,
        average_fee,
        deposited,
        balance,
        revenue,
    } = p.platform_query_app_info()?;

    assert_that(&user_stats.wins.value.u128()).is_equal_to(360_000);
    assert_that(&user_unclaimed.u128()).is_equal_to(0);
    assert_that(&average_fee.to_string().as_str()).is_equal_to("0.1");
    assert_that(&deposited.u128()).is_equal_to(1_000);
    assert_that(&balance.u128()).is_equal_to(15_000);
    assert_that(&revenue.total).is_equal_to(Int256::from(40_000));
    assert_that(&revenue.current).is_equal_to(Int256::from(14_000));

    p.platform_try_withdraw(ProjectAccount::Admin, None, None)?;

    let admin_balance_after = p.query_balance(ProjectAccount::Admin, &ProjectCoin::Om)?;
    let platform_balance_after = p.query_balance(p.get_platform_address(), &ProjectCoin::Om)?;
    let AppInfo {
        deposited,
        balance,
        revenue,
        ..
    } = p.platform_query_app_info()?;

    assert_that(&(admin_balance_after - admin_balance_before)).is_equal_to(40_000);
    assert_that(&(platform_balance_after - platform_balance_before)).is_equal_to(0);

    assert_that(&deposited.u128()).is_equal_to(0);
    assert_that(&balance.u128()).is_equal_to(0);
    assert_that(&revenue.total).is_equal_to(Int256::from(40_000));
    assert_that(&revenue.current).is_equal_to(Int256::from(0));

    Ok(())
}
