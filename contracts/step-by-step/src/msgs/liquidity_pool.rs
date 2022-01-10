use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary,
    Decimal, 
    Addr,
    CosmosMsg, WasmMsg,
    StdResult, 
};
use terra_cosmwasm::TerraMsgWrapper;
use terraswap::pair::ExecuteMsg as TerraswapExecuteMsg;
use terraswap::asset::Asset as TerraswapAsset;
use terraswap::asset::AssetInfo as TerraswapAssetInfo;
use crate::asset::{ Asset, AssetInfo };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LiquidityPoolSwapMsg {
    pool_addr: Addr,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>
}

fn asset_info_to_terraswap_info(asset: Asset) -> TerraswapAsset {
    match &asset.info {
        AssetInfo::Token { contract_addr } => TerraswapAsset { info: TerraswapAssetInfo::Token { contract_addr: contract_addr.to_string() }, amount: asset.amount },
        AssetInfo::NativeToken { denom } => TerraswapAsset { info: TerraswapAssetInfo::NativeToken { denom: denom.to_string() }, amount: asset.amount }
    }
}

impl LiquidityPoolSwapMsg {
    pub fn create_execution_message(&self, offer_asset: Asset, recipient: Addr) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
        return Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.pool_addr.to_string(),
            msg: to_binary(&TerraswapExecuteMsg::Swap {
                offer_asset: asset_info_to_terraswap_info(offer_asset),
                belief_price: self.belief_price,
                max_spread: self.max_spread,
                to: Some(recipient.to_string())
            })?,
            funds: vec![],
        }));
    }
}
