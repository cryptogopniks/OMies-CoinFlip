use cosmwasm_std::StdResult;
use cw_multi_test::Executor;

use cf_base::platform::msg::MigrateMsg;

use crate::helpers::suite::{core::Project, types::ProjectAccount};

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
fn default() -> StdResult<()> {
    let mut _p = Project::new();

    Ok(())
}
