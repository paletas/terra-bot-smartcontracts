use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{ CosmosMsg, Addr, StdResult };
use terra_cosmwasm::{ TerraMsgWrapper };

use crate::asset::{ Asset };
use crate::msgs::liquidity_pool::LiquidityPoolSwapMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StrategyStepOperation {
    LiquidityPoolSwapOperation {  
        msg: LiquidityPoolSwapMsg
    }
}

impl StrategyStepOperation {
    pub fn create_execution_message(&self, offer_asset: Asset, recipient: Addr) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
        match self {
            StrategyStepOperation::LiquidityPoolSwapOperation { msg } => msg.create_execution_message(offer_asset, recipient)
        }
    }
}