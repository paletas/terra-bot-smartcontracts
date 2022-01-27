use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::asset::{Asset, AssetInfo};
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, QuerierWrapper, QueryRequest, StdResult,
    Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;
use terra_cosmwasm::TerraMsgWrapper;
use terraswap::asset::Asset as TerraswapAsset;
use terraswap::asset::AssetInfo as TerraswapAssetInfo;
use terraswap::asset::PairInfo as TerraswapPairInfo;
use terraswap::factory::QueryMsg as FactoryQueryMsg;
use terraswap::pair::ExecuteMsg as PairExecuteMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LiquidityPoolSwapMsg {
    factory_addr: String,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
}

fn asset_to_terraswap_asset(asset: &Asset, amount: Option<Uint128>) -> TerraswapAsset {
    match &asset.info {
        AssetInfo::Token { .. } => TerraswapAsset {
            info: asset_info_to_terraswap_info(&asset.info),
            amount: amount.unwrap_or(asset.amount),
        },
        AssetInfo::NativeToken { .. } => TerraswapAsset {
            info: asset_info_to_terraswap_info(&asset.info),
            amount: amount.unwrap_or(asset.amount),
        },
    }
}

fn asset_info_to_terraswap_info(info: &AssetInfo) -> TerraswapAssetInfo {
    match info {
        AssetInfo::Token { contract_addr } => TerraswapAssetInfo::Token {
            contract_addr: contract_addr.to_string(),
        },
        AssetInfo::NativeToken { denom } => TerraswapAssetInfo::NativeToken {
            denom: denom.to_string(),
        },
    }
}

fn query_pair_info(
    querier: &QuerierWrapper,
    factory_contract: Addr,
    asset_infos: &[AssetInfo; 2],
) -> StdResult<TerraswapPairInfo> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: factory_contract.to_string(),
        msg: to_binary(&FactoryQueryMsg::Pair {
            asset_infos: [
                asset_info_to_terraswap_info(&asset_infos[0]),
                asset_info_to_terraswap_info(&asset_infos[1]),
            ],
        })?,
    }))
}

impl LiquidityPoolSwapMsg {
    pub fn create_execution_message(
        &self,
        deps: Deps,
        offer_asset: Asset,
        ask_asset_info: AssetInfo,
        to: String,
    ) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
        let factory_addr = deps.api.addr_validate(&self.factory_addr)?;
        let pair_info: TerraswapPairInfo = query_pair_info(
            &deps.querier,
            factory_addr,
            &[offer_asset.info.clone(), ask_asset_info],
        )?;

        match offer_asset.info.clone() {
            AssetInfo::NativeToken { denom } => {
                // deduct tax first
                let amount = offer_asset
                    .amount
                    .checked_sub(offer_asset.compute_tax(&deps.querier)?)?;

                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: pair_info.contract_addr,
                    funds: vec![Coin { denom, amount }],
                    msg: to_binary(&PairExecuteMsg::Swap {
                        offer_asset: asset_to_terraswap_asset(&offer_asset, Some(amount)),
                        belief_price: self.belief_price,
                        max_spread: self.max_spread,
                        to: Some(to),
                    })?,
                }))
            }
            AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: pair_info.contract_addr,
                    amount: offer_asset.amount,
                    msg: to_binary(&PairExecuteMsg::Swap {
                        offer_asset: asset_to_terraswap_asset(&offer_asset, None),
                        belief_price: self.belief_price,
                        max_spread: self.max_spread,
                        to: Some(to),
                    })?,
                })?,
            })),
        }
    }
}
