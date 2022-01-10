#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, Uint128, Addr, CosmosMsg, WasmMsg, QuerierWrapper};
use terra_cosmwasm::{TerraMsgWrapper};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StrategyStep, ConfigResponse};
use crate::state::{State, STATE};
use crate::asset::{ Asset, AssetInfo };
use crate::querier::{query_balance};

// version info for migration info
const CONTRACT_NAME: &str = "ThyBotIsThick.StepByStep";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.comission < 0 || msg.comission > 100 {
        return Err(ContractError::Std(StdError::generic_err("comission should be between 0 and 100")));
    }

    let state = State {
        owner: info.sender.clone(),
        comission: msg.comission
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<TerraMsgWrapper>> {
    match msg {
        ExecuteMsg::ExecuteStrategy { steps, minimum_receive } => execute_strategy(&deps.querier, deps.api, _env.contract.address, info.sender, steps, minimum_receive),
        ExecuteMsg::ExecuteStrategyStep { step } => execute_step(&deps.querier, _env.contract.address, step),
        ExecuteMsg::FinalizeStrategy { receiver, asset_info, initial_balance, minimum_receive } => finalize_strategy(&deps.querier, deps.api.addr_validate(receiver.as_str())?, asset_info, initial_balance, minimum_receive),
    }
}

fn execute_strategy(
    querier: &QuerierWrapper, 
    api: &dyn Api,
    contract_addr: Addr, 
    receiver: Addr, 
    steps: Vec<StrategyStep>, 
    minimum_receive: Uint128
) -> StdResult<Response<TerraMsgWrapper>> {
    let steps_len = steps.len();
    if steps_len == 0 {
        return Err(StdError::generic_err("must provide steps"));
    }
    
    let target_asset_info = steps.last().unwrap().get_to_asset();
    let receiver_addr = receiver.to_string();
        
    let mut step_index = 0;
    let mut messages: Vec<CosmosMsg<TerraMsgWrapper>> = steps
        .into_iter()
        .map(|op| {
            step_index += 1;
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteStrategyStep {
                    step: op
                })?,
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg<TerraMsgWrapper>>>>()?;

    // Execute minimum amount assertion
    let receiver_balance = target_asset_info.query_pool(querier, api, receiver)?;
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::FinalizeStrategy {
            receiver: receiver_addr,
            asset_info: target_asset_info,
            initial_balance: receiver_balance,
            minimum_receive: minimum_receive
        })?,
    }));

    return Ok(Response::new().add_messages(messages));
}

fn execute_step(
    querier: &QuerierWrapper, 
    contract_addr: Addr, 
    step: StrategyStep
) -> StdResult<Response<TerraMsgWrapper>> {
    let from_asset_info = &step.from_asset;

    let amount = query_balance(querier, contract_addr.clone(), from_asset_info.clone())?;
    let from_asset = Asset { info: from_asset_info.clone(), amount: amount };

    let msg = step.operation.create_execution_message(from_asset, contract_addr)?;

    return Ok(Response::new().add_message(msg));
}

fn finalize_strategy(
    querier: &QuerierWrapper, 
    receiver: Addr,
    target_asset_info: AssetInfo,
    initial_balance: Uint128, 
    minimum_receive: Uint128
) -> StdResult<Response<TerraMsgWrapper>> {
    let current_balance = query_balance(querier, receiver, target_asset_info)?;
    let swap_amount = current_balance.checked_sub(initial_balance)?;

    if swap_amount < minimum_receive {
        return Err(StdError::generic_err(format!(
            "assertion failed; minimum receive amount: {}, swap amount: {}",
            minimum_receive,
            swap_amount
        )));
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps, 
    _env: Env, 
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config { } => to_binary(&query_config(deps)?)
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let resp = ConfigResponse {
        comission: state.comission,
    };

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { comission: 6 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(6, value.comission);
    }
}
