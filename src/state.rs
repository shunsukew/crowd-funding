use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProjectInfo {
    pub title: String,
    pub description: String,
    pub project_owner: Addr,
    // only token defined here can be bond to the contract
    // in other words, which token can be contributed
    pub denom: String,
    pub target_amount: Uint128,
    pub deadline: u64,

    // current amout of denom token contributed
    pub current_amount: Uint128,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum Status {
    Ongoing,
    Succeeded,
    Expired,
}

pub const PROJECT_INFO: Item<ProjectInfo> = Item::new("project_info");
pub const CONTRIBUTIONS: Map<&Addr, Uint128> = Map::new("contributions");
