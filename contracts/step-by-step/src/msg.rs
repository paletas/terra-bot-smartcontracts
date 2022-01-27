use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

use crate::asset::AssetInfo;
use crate::operations::StrategyStepOperation;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub comission: i16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    ExecuteStrategy {
        steps: Vec<StrategyStep>,
        minimum_receive: Uint128,
    },
    /* INTERNAL USE ONLY */
    ExecuteStrategyStep {
        step: StrategyStep,
        to: String,
    },
    /* INTERNAL USE ONLY */
    FinalizeStrategy {
        receiver: String,
        asset_info: AssetInfo,
        initial_balance: Uint128,
        minimum_receive: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    ExecuteStrategy {
        steps: Vec<StrategyStep>,
        minimum_receive: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StrategyStep {
    pub from_asset: AssetInfo,
    pub to_asset: AssetInfo,
    pub operation: StrategyStepOperation,
}

impl StrategyStep {
    pub fn get_from_asset(&self) -> AssetInfo {
        return self.from_asset.clone();
    }

    pub fn get_to_asset(&self) -> AssetInfo {
        return self.to_asset.clone();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub comission: i16,
}
