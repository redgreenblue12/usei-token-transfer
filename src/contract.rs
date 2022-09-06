#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetOwnerResponse, InstantiateMsg, QueryMsg};
use crate::state::{config, config_read, Config};

/*
const CONTRACT_NAME: &str = "crates.io:usei-transfer-tokens";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
 */

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Get the owner either from the message that the creator instantiates with, if they
    // specify an owner.
    // Alternatively, set the owner as the creator.
    let owner: Result<Addr, StdError> = match msg.owner {
        Some(explicit_owner) => Ok(deps.api.addr_validate(&explicit_owner)?),
        None => Ok(info.sender),
    };

    // Instantiate the contract.
    let config_state: Config = Config { owner: owner? };
    config(deps.storage).save(&config_state)?;
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
        ExecuteMsg::SendCoins { addr1, addr2 } => execute_send_coins(deps, env, info, addr1, addr2),
    }
}

pub fn execute_send_coins(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addr1: String,
    addr2: String,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => query_get_owner(deps, env, msg),
        QueryMsg::Config {} => query_config(deps, env, msg),
    }
}

pub fn query_get_owner(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let config_data = config_read(deps.storage).load()?;
    let resp = GetOwnerResponse {
        owner: String::from(config_data.owner),
    };
    to_binary(&resp)
}

pub fn query_config(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
  let config_data = config_read(deps.storage).load()?;
  to_binary(&config_data)
}

#[cfg(test)]
mod tests {}
