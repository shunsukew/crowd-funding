#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cosmwasm_std::{Addr, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetContributionResponse, GetProjectInfoResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{ProjectInfo, Status, CONTRIBUTIONS, PROJECT_INFO};

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
    let project_info = ProjectInfo {
        title: msg.title,
        description: msg.description,
        project_owner: info.sender,
        denom: msg.target_amount.denom,
        target_amount: msg.target_amount.amount,
        end_time: msg.end_time,
        current_amount: Uint128::zero(),
        status: Status::Ongoing,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    PROJECT_INFO.save(deps.storage, &project_info)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Contribute {} => try_contribute(deps, env, info),
        ExecuteMsg::Withdraw {} => try_withdraw(deps, env, info),
        ExecuteMsg::Refund {} => try_refund(deps, env, info),
    }
}

pub fn try_contribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
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
        .find(|x| x.denom == project_info.denom)
        .ok_or_else(|| ContractError::CustomError {
            val: format!("Only denom {} accepted", &project_info.denom),
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

pub fn try_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
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

    Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: project_info.project_owner.into(),
        amount: vec![Coin::new(
            project_info.current_amount.into(),
            project_info.denom,
        )],
    })))
}

pub fn try_refund(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
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
    match result {
        Some(amount) => {
            CONTRIBUTIONS.remove(deps.storage, &info.sender);

            Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.into(),
                amount: vec![Coin::new(amount.into(), project_info.denom)],
            })))
        }
        None => {
            return Err(ContractError::CustomError {
                val: "no contribution found".into(),
            })
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProjectInfo {} => to_binary(&query_project_info(deps, env)?),
        QueryMsg::GetContribution { address } => to_binary(&query_contribution(deps, address)?),
    }
}

fn query_project_info(deps: Deps, env: Env) -> StdResult<GetProjectInfoResponse> {
    let mut project_info = PROJECT_INFO.load(deps.storage)?;
    let now: u64 = env.block.time.clone().seconds();
    if project_info.end_time < now && project_info.current_amount < project_info.target_amount {
        project_info.status = Status::Failed;
    }

    Ok(GetProjectInfoResponse {
        title: project_info.title,
        description: project_info.description,
        project_owner: project_info.project_owner,
        denom: project_info.denom,
        target_amount: project_info.target_amount,
        end_time: project_info.end_time,
        current_amount: project_info.current_amount,
        status: project_info.status,
    })
}

fn query_contribution(deps: Deps, address: Addr) -> StdResult<GetContributionResponse> {
    let project_info = PROJECT_INFO.load(deps.storage)?;
    let contribution = CONTRIBUTIONS.may_load(deps.storage, &address)?;
    let contributed_amount = match contribution {
        Some(amount) => amount,
        None => Uint128::zero(),
    };
    Ok(GetContributionResponse {
        denom: project_info.denom,
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
