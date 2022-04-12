#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point};
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Addr, Uint128,
};
use cw2::set_contract_version;
use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetContributionResponse, GetProjectInfoResponse, InstantiateMsg, QueryMsg, Token
};
use crate::state::{ProjectInfo, Status, TokenConfig, CONTRIBUTIONS, PROJECT_INFO, TOKEN_CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crowd-funding";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    match msg.token {
        Token::Native { denom } => {
            TOKEN_CONFIG.save(deps.storage, &TokenConfig::Native{
                denom,
            })?;
        },
        Token::CW20 { addr} => {
            TOKEN_CONFIG.save(deps.storage, &TokenConfig::CW20{
                addr,
            })?;
        },
    }
    
    let project_info = ProjectInfo {
        title: msg.title,
        description: msg.description,
        project_owner: info.sender.clone(),
        target_amount: msg.target_amount,
        end_time: msg.end_time,
        current_amount: Uint128::zero(),
        status: Status::Ongoing,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    PROJECT_INFO.save(deps.storage, &project_info)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // contribute msg only when token config set to native
        ExecuteMsg::Contribute {} => try_contribute(deps, env, info),
        // recieve msg only when token config set to cw20.
        // To contribute, user need to send cw20 token to this contract address, then recieve msg is hooked.
        ExecuteMsg::Receive(msg) => try_recieve_and_contribute(deps, env, info, msg),
        ExecuteMsg::Withdraw {} => try_withdraw(deps, env, info),
        ExecuteMsg::Refund {} => try_refund(deps, env, info),
    }
}

pub fn try_contribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let token_config = TOKEN_CONFIG.load(deps.storage)?;

    let config_denom: String;
    match token_config {
        TokenConfig::Native{ denom } => {
            config_denom = denom;
        }
        TokenConfig::CW20{ addr: _ } => {
            return Err(ContractError::CustomError {
                val: "contribute msg is available only when token config set to Native".into(),
            });
        }
    }

    let mut project_info = PROJECT_INFO.load(deps.storage)?;
    if info.sender == project_info.project_owner {
        return Err(ContractError::CustomError {
            val: "project owner cannot contribute".into(),
        });
    }

    let now: u64 = env.block.time.clone().seconds();
    if project_info.end_time <= now {
        return Err(ContractError::CustomError {
            val: "end_time already exceeded".into(),
        });
    }

    // only the same denom is acceptable
    let contribute = info
        .funds
        .iter()
        .find(|x| x.denom == config_denom)
        .ok_or_else(|| ContractError::CustomError {
            val: format!("Only denom {} accepted", &config_denom),
        })?;

    let contributed_amount = contribute.amount;

    // update current amount
    project_info.current_amount += contributed_amount;
    if project_info.target_amount <= project_info.current_amount
        && project_info.status != Status::Succeeded
    {
        project_info.status = Status::Succeeded;
    }
    PROJECT_INFO.save(deps.storage, &project_info)?;

    // update contribution map
    CONTRIBUTIONS.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + contributed_amount)
    })?;

    let res = Response::new()
        .add_attribute("action", "contribute")
        .add_attribute("denom", &contribute.denom)
        .add_attribute("amount", contribute.amount);

    Ok(res)
}

pub fn try_recieve_and_contribute (deps: DepsMut, env: Env, info: MessageInfo, wrapped: Cw20ReceiveMsg) -> Result<Response, ContractError> {
    let token_config = TOKEN_CONFIG.load(deps.storage)?;

    let config_cw20_addr: Addr;
    match token_config {
        TokenConfig::Native{ denom: _ } => {
            return Err(ContractError::CustomError {
                val: "contribute msg is available only when token config set to Native".into(),
            });
        }
        TokenConfig::CW20{ addr } => {
            config_cw20_addr = addr;
        }
    }

    let mut project_info = PROJECT_INFO.load(deps.storage)?;
    // wrapped.sender is original msg executor
    if wrapped.sender == project_info.project_owner {
        return Err(ContractError::CustomError {
            val: "project owner cannot contribute".into(),
        });
    }

    let now: u64 = env.block.time.clone().seconds();
    if project_info.end_time <= now {
        return Err(ContractError::CustomError {
            val: "end_time already exceeded".into(),
        });
    }

    // info.sender is cw20 contract address
    // only configured cw20 token acceptable
    if info.sender != config_cw20_addr {
        return Err(ContractError::CustomError {
            val: "wrong cw20 token recieved".into(),
        });
    }

    // wrapped.amount is amount of cw20 which is sent
    let contributed_amount = wrapped.amount;

    // update current amount
    project_info.current_amount += contributed_amount;
    if project_info.target_amount <= project_info.current_amount
        && project_info.status != Status::Succeeded
    {
        project_info.status = Status::Succeeded;
    }
    PROJECT_INFO.save(deps.storage, &project_info)?;

    // update contribution map
    CONTRIBUTIONS.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + contributed_amount)
    })?;

    let res = Response::new()
        .add_attribute("action", "contribute")
        .add_attribute("cw20_address", &config_cw20_addr)
        .add_attribute("amount", contributed_amount);

    Ok(res)
}

pub fn try_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let token_config = TOKEN_CONFIG.load(deps.storage)?;

    let project_info = PROJECT_INFO.load(deps.storage)?;
    if info.sender != project_info.project_owner {
        return Err(ContractError::CustomError {
            val: "only project owner can withdraw".into(),
        });
    }

    let now: u64 = env.block.time.clone().seconds();
    if now < project_info.end_time {
        return Err(ContractError::CustomError {
            val: "project not ended".into(),
        });
    }

    if project_info.status != Status::Succeeded {
        return Err(ContractError::CustomError {
            val: "project not succeeded".into(),
        });
    }

    let msg: CosmosMsg;
    match token_config {
        TokenConfig::Native{ denom } => {
            msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: project_info.project_owner.into(),
                amount: vec![Coin::new(
                    project_info.current_amount.into(),
                    denom,
                )],
            });
        },
        TokenConfig::CW20{ addr } => {
            let cw20 = Cw20Contract(addr);
            msg = cw20.call(Cw20ExecuteMsg::Transfer {
                recipient: project_info.project_owner.into(),
                amount: project_info.current_amount.into(),
            })?;
        },
    }

    Ok(Response::new()
        .add_message(msg))
}

pub fn try_refund(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let token_config = TOKEN_CONFIG.load(deps.storage)?;
    let mut project_info = PROJECT_INFO.load(deps.storage)?;

    let now: u64 = env.block.time.clone().seconds();
    if now < project_info.end_time {
        return Err(ContractError::CustomError {
            val: "project not ended".into(),
        });
    }

    if project_info.current_amount < project_info.target_amount {
        project_info.status = Status::Failed;
    } else {
        project_info.status = Status::Succeeded;
    }

    if project_info.status != Status::Failed {
        return Err(ContractError::CustomError {
            val: "project not failed".into(),
        });
    }

    let result = CONTRIBUTIONS.may_load(deps.storage, &info.sender)?;
    if result.is_none() {
        return Err(ContractError::CustomError {
            val: "no contribution found".into(),
        })
    }
    // result is not None here
    let refund_amount = result.unwrap();

    let msg: CosmosMsg;
    match token_config {
        TokenConfig::Native{ denom } => {
            msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.into(),
                amount: vec![Coin::new(
                    refund_amount.into(),
                    denom,
                )],
            });
        },
        TokenConfig::CW20{ addr } => {
            let cw20 = Cw20Contract(addr);
            msg = cw20.call(Cw20ExecuteMsg::Transfer {
                recipient: info.sender.into(),
                amount: refund_amount.into(),
            })?;
        },
    }

    Ok(Response::new()
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProjectInfo {} => to_binary(&query_project_info(deps, env)?),
        QueryMsg::GetContribution { address } => to_binary(&query_contribution(deps, address)?),
    }
}

fn query_project_info(deps: Deps, env: Env) -> StdResult<GetProjectInfoResponse> {
    let token_config = TOKEN_CONFIG.load(deps.storage)?;
    let token = match token_config {
        TokenConfig::Native{ denom } => {
            Token::Native{denom}
        },
        TokenConfig::CW20{ addr } => {
            Token::CW20{addr}
        }
    };
    let mut project_info = PROJECT_INFO.load(deps.storage)?;
    let now: u64 = env.block.time.clone().seconds();
    if project_info.end_time < now && project_info.current_amount < project_info.target_amount {
        project_info.status = Status::Failed;
    }

    Ok(GetProjectInfoResponse {
        title: project_info.title,
        description: project_info.description,
        project_owner: project_info.project_owner,
        token: token,
        target_amount: project_info.target_amount,
        end_time: project_info.end_time,
        current_amount: project_info.current_amount,
        status: project_info.status,
    })
}

fn query_contribution(deps: Deps, address: Addr) -> StdResult<GetContributionResponse> {
    let token_config = TOKEN_CONFIG.load(deps.storage)?;
    let token = match token_config {
        TokenConfig::Native{ denom } => {
            Token::Native{denom}
        },
        TokenConfig::CW20{ addr } => {
            Token::CW20{addr}
        }
    };

    let contribution = CONTRIBUTIONS.may_load(deps.storage, &address)?;
    let contributed_amount = match contribution {
        Some(amount) => amount,
        None => Uint128::zero(),
    };
    Ok(GetContributionResponse {
        token: token,
        amount: contributed_amount,
    })
}

#[cfg(test)]
mod tests {
    //use super::*;
    //use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    //use cosmwasm_std::{coins, from_binary};

    //#[test]
    //fn proper_initialization() {
    //let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //let msg = InstantiateMsg { count: 17 };
    //let info = mock_info("creator", &coins(1000, "earth"));

    //// we can just call .unwrap() to assert this was a success
    //let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //assert_eq!(0, res.messages.len());

    //// it worked, let's query the state
    //let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //let value: CountResponse = from_binary(&res).unwrap();
    //assert_eq!(17, value.count);
    //}

    //#[test]
    //fn increment() {
    //let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //let msg = InstantiateMsg { count: 17 };
    //let info = mock_info("creator", &coins(2, "token"));
    //let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //// beneficiary can release it
    //let info = mock_info("anyone", &coins(2, "token"));
    //let msg = ExecuteMsg::Increment {};
    //let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //// should increase counter by 1
    //let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //let value: CountResponse = from_binary(&res).unwrap();
    //assert_eq!(18, value.count);
    //}

    //#[test]
    //fn reset() {
    //let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //let msg = InstantiateMsg { count: 17 };
    //let info = mock_info("creator", &coins(2, "token"));
    //let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //// beneficiary can release it
    //let unauth_info = mock_info("anyone", &coins(2, "token"));
    //let msg = ExecuteMsg::Reset { count: 5 };
    //let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //match res {
    //Err(ContractError::Unauthorized {}) => {}
    //_ => panic!("Must return unauthorized error"),
    //}

    //// only the original creator can reset the counter
    //let auth_info = mock_info("creator", &coins(2, "token"));
    //let msg = ExecuteMsg::Reset { count: 5 };
    //let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //// should now be 5
    //let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //let value: CountResponse = from_binary(&res).unwrap();
    //assert_eq!(5, value.count);
    //}
}
