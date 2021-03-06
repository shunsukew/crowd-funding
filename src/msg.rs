use crate::state::Status;
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Token {
    Native {
        denom: String,
    },
    CW20 {
        addr: Addr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // project title
    pub title: String,
    // project description
    pub description: String,
    // CW20 token address
    pub token: Token,
    // project owner
    // pub project_owner: Addr,
    // target amount project owner want to raise
    pub target_amount: Uint128,
    /// When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    /// block time exceeds this value, the crowd funding is Failed.
    /// Once an project is Failed, raised amount coins can be returned to the original funder (via "refund").
    pub end_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // anyone can contribute coins to a project with native token
    Contribute {},
    // anyone can contribute coins to a project with cw20
    Receive(Cw20ReceiveMsg),
    // only project owner can withdraw raised funds
    Withdraw {},
    // contributors can execute refund after the end_time
    // if the raised amount didn't satisfy target amount before end_time
    Refund {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProjectInfo {},
    GetContribution { address: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetProjectInfoResponse {
    pub title: String,
    pub description: String,
    pub project_owner: Addr,
    pub token: Token,
    pub target_amount: Uint128,
    pub end_time: u64,

    pub current_amount: Uint128,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetContributionResponse {
    pub token: Token,
    pub amount: Uint128,
}
