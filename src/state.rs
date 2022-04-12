use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

// Token config is immutable once contract created
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenConfig {
    // when is_native true
    Native { denom: String },
    // when is_native false. this managing cw20 token
    CW20 { addr: Addr },
}

// Potentially mutable data, depends on the crowdfunding specification
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProjectInfo {
    pub title: String,
    pub description: String,
    pub project_owner: Addr,
    // target amount of token
    pub target_amount: Uint128,
    // when crowd funding project ends
    pub end_time: u64,

    // current amout of denom token contributed
    pub current_amount: Uint128,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum Status {
    Ongoing,
    Succeeded,
    Failed,
}

pub const TOKEN_CONFIG: Item<TokenConfig> = Item::new("token_config");
pub const PROJECT_INFO: Item<ProjectInfo> = Item::new("project_info");
pub const CONTRIBUTIONS: Map<&Addr, Uint128> = Map::new("contributions");
