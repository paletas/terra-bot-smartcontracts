#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use terra_cosmwasm::TerraMsgWrapper;

use crate::asset::{Asset, AssetInfo};
use crate::error::ContractError;
use crate::msg::{ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StrategyStep};
use crate::querier::query_balance;
use crate::state::{State, STATE};

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
        return Err(ContractError::Std(StdError::generic_err(
            "comission should be between 0 and 100",
        )));
    }

    let state = State {
        owner: info.sender.clone(),
        comission: msg.comission,
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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, _env, info, msg),
        ExecuteMsg::ExecuteStrategy {
            steps,
            minimum_receive,
        } => execute_strategy(deps, _env, info, steps, minimum_receive),
        ExecuteMsg::ExecuteStrategyStep { step, to } => execute_step(deps, _env, info, step, to),
        ExecuteMsg::FinalizeStrategy {
            receiver,
            asset_info,
            initial_balance,
            minimum_receive,
        } => finalize_strategy(
            deps.as_ref(),
            _env,
            info,
            deps.api.addr_validate(receiver.as_str())?,
            asset_info,
            initial_balance,
            minimum_receive,
        ),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response<TerraMsgWrapper>> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::ExecuteStrategy {
            steps,
            minimum_receive,
        } => execute_strategy(deps, _env, info, steps, minimum_receive),
    }
}

fn execute_strategy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    steps: Vec<StrategyStep>,
    minimum_receive: Uint128,
) -> StdResult<Response<TerraMsgWrapper>> {
    let steps_len = steps.len();
    if steps_len == 0 {
        return Err(StdError::generic_err("must provide steps"));
    }

    let to = info.sender;
    let from_asset_info = steps.first().unwrap().get_from_asset();
    let target_asset_info = steps.last().unwrap().get_to_asset();

    if from_asset_info.equal(&target_asset_info) {
        let current_amount =
            query_balance(&deps.querier, env.contract.address.clone(), from_asset_info)?;

        if current_amount < minimum_receive {
            return Err(StdError::generic_err(format!(
                "assertion failed; receive amount: {} is lower than minimum amount: {}",
                current_amount, minimum_receive
            )));
        }
    }

    let mut step_index = 0;
    let mut messages: Vec<CosmosMsg<TerraMsgWrapper>> = steps
        .into_iter()
        .map(|op| {
            step_index += 1;
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteStrategyStep {
                    step: op,
                    to: if step_index == steps_len {
                        to.to_string()
                    } else {
                        env.contract.address.to_string()
                    },
                })?,
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg<TerraMsgWrapper>>>>()?;

    // Execute minimum amount assertion
    let receiver_balance = target_asset_info.query_pool(&deps.querier, deps.api, to.clone())?;
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::FinalizeStrategy {
            receiver: to.to_string(),
            asset_info: target_asset_info,
            initial_balance: receiver_balance,
            minimum_receive: minimum_receive,
        })?,
    }));

    Ok(Response::new().add_messages(messages))
}

fn execute_step(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    step: StrategyStep,
    to: String,
) -> StdResult<Response<TerraMsgWrapper>> {
    if env.contract.address != info.sender {
        return Err(StdError::generic_err(format!(
            "unauthorized step; expected caller: {}, caller: {}",
            env.contract.address, info.sender
        )));
    }

    let contract_addr = env.contract.address;

    let amount = query_balance(&deps.querier, contract_addr, step.get_from_asset())?;
    let from_asset = Asset {
        info: step.get_from_asset(),
        amount: amount,
    };
    let to_asset_info = step.get_to_asset();

    let msg =
        step.operation
            .create_execution_message(deps.as_ref(), from_asset, to_asset_info, to)?;

    return Ok(Response::new().add_message(msg));
}

fn finalize_strategy(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    receiver: Addr,
    target_asset_info: AssetInfo,
    initial_balance: Uint128,
    minimum_receive: Uint128,
) -> StdResult<Response<TerraMsgWrapper>> {
    if env.contract.address != info.sender {
        return Err(StdError::generic_err(format!(
            "unauthorized finalize; expected caller: {}, caller: {}",
            env.contract.address, info.sender
        )));
    }

    let current_balance = query_balance(&deps.querier, receiver, target_asset_info.clone())?;
    let swap_amount = current_balance.checked_sub(initial_balance)?;

    if swap_amount < minimum_receive {
        return Err(StdError::generic_err(format!(
            "assertion failed; minimum receive amount: {}, swap amount: {}",
            minimum_receive, swap_amount
        )));
    }

    Ok(Response::default()
        .add_attribute("initial_balance", initial_balance)
        .add_attribute("final_balance", current_balance)
        .add_attribute("target_asset", target_asset_info.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
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
