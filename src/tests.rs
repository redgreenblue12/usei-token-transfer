#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, Addr, Api, Coin, Deps, DepsMut};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{GetOwnerResponse, InstantiateMsg, QueryMsg};
    use crate::state::Config;

    fn assert_config_state(deps: Deps, expected: Config) {
        let res = query(deps, mock_env(), QueryMsg::Config {}).unwrap();
        let value: Config = from_binary(&res).unwrap();
        assert_eq!(value, expected);
    }

    fn mock_init_no_owner_specified(deps: DepsMut) {
        let msg = InstantiateMsg { owner: None };

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    fn mock_init_owner_specified(deps: DepsMut, owner: String) {
        let msg = InstantiateMsg { owner: Some(owner) };

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    #[test]
    fn create_contract_with_implict_owner() {
        let mut deps = mock_dependencies();
        mock_init_no_owner_specified(deps.as_mut());
        assert_config_state(
            deps.as_ref(),
            Config {
                owner: Addr::unchecked("creator"),
            },
        )
    }

    #[test]
    fn create_contract_with_explicit_owner() {
        let mut deps = mock_dependencies();
        mock_init_owner_specified(deps.as_mut(), String::from("someone"));
        assert_config_state(
            deps.as_ref(),
            Config {
                owner: Addr::unchecked("someone"),
            },
        )
    }

    #[test]
    fn get_contract_owner() {
        let mut deps = mock_dependencies();
        let owner = String::from("someone");
        mock_init_owner_specified(deps.as_mut(), owner);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let data: GetOwnerResponse = from_binary(&res).unwrap();

        assert_eq!(data.owner, "someone");
    }
}
