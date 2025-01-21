use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

use super::types::{Config, FlipStats, TransferAdminState, UserInfo};

pub const CONTRACT_NAME: &str = "CryptoGopniks: OMies CoinFlip";

pub const MEAN_WEIGHT: &str = "0.5";
pub const DENOM_DEFAULT: &str = "uom";
pub const FLIP_COOLDOWN: u64 = 1;
pub const TRANSFER_ADMIN_TIMEOUT: u64 = 7 * 24 * 3_600;

pub const IS_PAUSED: Item<bool> = Item::new("is_paused");
pub const TRANSFER_ADMIN_STATE: Item<TransferAdminState> = Item::new("transfer_admin_state");
pub const CONFIG: Item<Config> = Item::new("config");
pub const NORMALIZED_DECIMAL: Item<Decimal> = Item::new("normalized_decimal");
pub const FLIP_STATS: Item<FlipStats> = Item::new("flip_stats");

pub const USERS: Map<&Addr, UserInfo> = Map::new("users");
