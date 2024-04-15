use crate::bridge::contract_clients::config::Config;
use crate::bridge::contract_clients::eth_bridge::BridgeDeployable;
use crate::bridge::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::bridge::contract_clients::token_bridge::StarknetTokenBridge;
use crate::ArgConfig;
use anyhow::Ok;
use sp_core::{H160, U256};
use starknet_core::{
    types::{BlockId, BlockTag, FunctionCall},
    utils::get_selector_from_name,
};
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

pub async fn deploy_erc20_bridge(
    clients: &Config,
    arg_config: ArgConfig,
    core_contract: &StarknetSovereignContract,
) -> Result<(StarknetTokenBridge, FieldElement, FieldElement), anyhow::Error> {
    let token_bridge = StarknetTokenBridge::deploy(core_contract.client().clone()).await;

    log::debug!("Token Bridge Deployment Successful [✅]");
    log::debug!(
        "[🚀] Token Bridge Address : {:?}",
        token_bridge.bridge_address()
    );
    log::debug!("[🚀] ERC 20 Token Address : {:?}", token_bridge.address());

    let l2_bridge_address = StarknetTokenBridge::deploy_l2_contracts(
        &clients.provider_l2(),
        &arg_config.rollup_priv_key,
        &arg_config.l2_deployer_address,
    )
    .await;

    log::debug!("L2 Token Bridge Deployment Successful [✅]");
    log::debug!("[🚀] L2 Token Bridge Address : {:?}", l2_bridge_address);

    token_bridge.initialize(core_contract.address()).await;
    token_bridge
        .setup_l2_bridge(
            &clients.provider_l2(),
            l2_bridge_address,
            &arg_config.rollup_priv_key,
            &arg_config.l2_deployer_address,
        )
        .await;
    token_bridge
        .setup_l1_bridge(
            H160::from_str(&arg_config.l1_deployer_address).unwrap(),
            l2_bridge_address,
            U256::from_dec_str("100000000000000").unwrap(),
        )
        .await;

    sleep(Duration::from_secs(
        arg_config.l1_wait_time.parse().unwrap(),
    ))
    .await;
    sleep(Duration::from_millis(60000)).await;

    let l2_erc20_token_address = get_l2_token_address(
        &clients.provider_l2(),
        &l2_bridge_address,
        &token_bridge.address(),
    )
    .await;
    log::debug!(
        "[🚀] L2 ERC 20 Token Address : {:?}",
        l2_erc20_token_address
    );

    Ok((token_bridge, l2_bridge_address, l2_erc20_token_address))
}

async fn get_l2_token_address(
    rpc_provider_l2: &JsonRpcClient<HttpTransport>,
    l2_bridge_address: &FieldElement,
    l1_erc_20_address: &H160,
) -> FieldElement {
    let l2_address = rpc_provider_l2
        .call(
            FunctionCall {
                contract_address: l2_bridge_address.clone(),
                entry_point_selector: get_selector_from_name("get_l2_token").unwrap(),
                calldata: vec![
                    FieldElement::from_byte_slice_be(l1_erc_20_address.as_bytes()).unwrap(),
                ],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap()[0];

    l2_address
}
