use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Response, Deps, StdResult};
use terra_cosmwasm::TerraMsgWrapper;

use crate::asset::{Asset, AssetInfo};
use crate::msgs::liquidity_pool::LiquidityPoolSwapMsg;
use crate::msgs::market::MarketSwapMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StrategyStepOperation {
    LiquidityPoolSwapOperation { msg: LiquidityPoolSwapMsg },
    MarketSwapOperation { msg: MarketSwapMsg }
}

impl StrategyStepOperation {
    pub fn create_execution_message(
        &self,
        deps: Deps,
        offer_asset: Asset,
        ask_asset_info: AssetInfo,
        to: Option<String>,
    ) -> StdResult<Response<TerraMsgWrapper>> {
        match self {
            StrategyStepOperation::LiquidityPoolSwapOperation { msg } => {
                msg.create_execution_message(deps, offer_asset, ask_asset_info, to)
            },
            StrategyStepOperation::MarketSwapOperation { msg } => {
                msg.create_execution_message(deps, offer_asset, ask_asset_info, to)
            }
        }
    }
}
