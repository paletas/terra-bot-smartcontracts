use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CosmosMsg, Deps, StdResult};
use terra_cosmwasm::TerraMsgWrapper;

use crate::asset::{Asset, AssetInfo};
use crate::msgs::liquidity_pool::LiquidityPoolSwapMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StrategyStepOperation {
    LiquidityPoolSwapOperation { msg: LiquidityPoolSwapMsg },
}

impl StrategyStepOperation {
    pub fn create_execution_message(
        &self,
        deps: Deps,
        offer_asset: Asset,
        ask_asset_info: AssetInfo,
        to: String,
    ) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
        match self {
            StrategyStepOperation::LiquidityPoolSwapOperation { msg } => {
                msg.create_execution_message(deps, offer_asset, ask_asset_info, to)
            }
        }
    }
}
