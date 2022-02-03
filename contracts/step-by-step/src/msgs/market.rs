use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::asset::{Asset, AssetInfo};
use cosmwasm_std::{Coin, Deps, StdResult, StdError, Response, CosmosMsg};
use terra_cosmwasm::{create_swap_msg, create_swap_send_msg, TerraMsgWrapper};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarketSwapMsg {
}

impl MarketSwapMsg {
    pub fn create_execution_message(
        &self,
        deps: Deps,
        offer_asset: Asset,
        ask_asset_info: AssetInfo,
        to: Option<String>,
    ) -> StdResult<Response<TerraMsgWrapper>> {
        if matches!(offer_asset.info, AssetInfo::Token { .. }) {
            return Err(StdError::generic_err("assertion failed; custom tokens not supported"));
        }

        if matches!(ask_asset_info, AssetInfo::Token { .. }) {
            return Err(StdError::generic_err("assertion failed; custom tokens not supported"));
        }

        let messages: Vec<CosmosMsg<TerraMsgWrapper>> = match offer_asset.info.clone() {
            AssetInfo::NativeToken { denom: offer_denom } => {
                match ask_asset_info.clone() {
                    AssetInfo::NativeToken { denom: ask_denom } => {
                        if let Some(to) = to {
                            // if the operation is last, and requires send
                            // deduct tax from the offer_coin
                            let amount = offer_asset
                                .amount
                                .checked_sub(offer_asset.compute_tax(&deps.querier)?)?;
        
                            vec![create_swap_send_msg(
                                to,
                                Coin {
                                    denom: offer_denom,
                                    amount: amount,
                                },
                                ask_denom,
                            )]
                        } else {
                            vec![create_swap_msg(
                                Coin {
                                    denom: offer_denom,
                                    amount: offer_asset.amount,
                                },
                                ask_denom,
                            )]
                        }
                    },
                    AssetInfo::Token { .. } => vec![]
                }
            },
            AssetInfo::Token { .. } => vec![]
        };

        Ok(Response::new().add_messages(messages))
    }
}