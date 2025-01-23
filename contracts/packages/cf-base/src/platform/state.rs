use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

use super::types::{AppInfo, Config, TransferAdminState, UserInfo};

pub const CONTRACT_NAME: &str = "CryptoGopniks: OMies CoinFlip";

pub const SEED: &str = "8888";
pub const BET_MIN: u64 = 1_000_000;
pub const BET_MAX: u64 = 20_000_000;
pub const DENOM: &str = "uom";
pub const PLATFORM_FEE: &str = "0.1";
pub const FLIP_COOLDOWN: u64 = 3;
pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3_600;

pub const IS_PAUSED: Item<bool> = Item::new("is_paused");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer_admin_state");
pub const CONFIG: Item<Config> = Item::new("config");
pub const NORMALIZED_DECIMAL: Item<Decimal> = Item::new("normalized_decimal");
pub const APP_INFO: Item<AppInfo> = Item::new("app_info");

pub const USERS: Map<&Addr, UserInfo> = Map::new("users");
