#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage,
};
use cosmwasm_storage::Bucket;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetAccountBalanceResponse, GetOwnerResponse, InstantiateMsg, QueryMsg,
};
use crate::state::{config, config_read, resolver, resolver_read, AccountBalance, Config};

pub static COIN_DENOM: &str = "usei";

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
        ExecuteMsg::SendCoins {
            dest_addr1,
            dest_addr2,
        } => execute_send_coins(deps, env, info, dest_addr1, dest_addr2),
    }
}

pub fn execute_send_coins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    dest_addr1: String,
    dest_addr2: String,
) -> Result<Response, ContractError> {
    let valid_dest_addr1 = deps.api.addr_validate(&dest_addr1)?;
    let valid_dest_addr2 = deps.api.addr_validate(&dest_addr2)?;
    let total_coin_quantity = get_coin_quantity_sent_in_message(info);
    let split_coin_quantity = total_coin_quantity / 2;
    send_coins_to_address(
        &mut resolver(deps.storage),
        valid_dest_addr1,
        split_coin_quantity,
    )?;
    send_coins_to_address(
        &mut resolver(deps.storage),
        valid_dest_addr2,
        split_coin_quantity,
    )?;
    Ok(Response::default())
}

pub fn send_coins_to_address(
    resolver: &mut Bucket<AccountBalance>,
    valid_dest_addr: Addr,
    coin_quantity: u128,
) -> Result<AccountBalance, ContractError> {
    let key = valid_dest_addr.as_bytes();
    resolver.update(key, |account_balance: Option<AccountBalance>| {
        if let Some(mut account_balance) = account_balance {
            account_balance.balance = account_balance.balance + coin_quantity;
            Ok::<AccountBalance, ContractError>(account_balance)
        } else {
            let new_balance = AccountBalance {
                address: valid_dest_addr.clone(),
                balance: coin_quantity,
            };
            Ok(new_balance)
        }
    })
}

pub fn get_coin_quantity_sent_in_message(info: MessageInfo) -> u128 {
    let coins_sent = info.funds.iter().find(|coin| coin.denom == COIN_DENOM);
    match coins_sent {
        Some(coins) => coins.amount.u128(),
        None => 0,
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => query_get_owner(deps, env, msg),
        QueryMsg::Config {} => query_config(deps, env, msg),
        QueryMsg::GetAccountBalance { address } => query_get_account_balance(deps, env, address),
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

pub fn query_get_account_balance(deps: Deps, _env: Env, address: String) -> StdResult<Binary> {
    let valid_address = deps.api.addr_validate(&address)?;
    let key = valid_address.as_bytes();
    let resp = match resolver_read(deps.storage).may_load(key)? {
        Some(ab) => GetAccountBalanceResponse {
            // TODO: In code review, I think it's arguable that it's better to have
            // ab.address.to_string() for sanity, instead of slightly optimizing for gas here
            address: address,
            balance: ab.balance,
        },
        None => GetAccountBalanceResponse {
            address: address,
            balance: 0,
        },
    };
    to_binary(&resp)
}
