use std::future::Future;
use std::sync::Arc;
use assert_matches::assert_matches;
use async_trait::async_trait;
use starknet_accounts::{Account, Call, ConnectedAccount, Declaration, Execution, LegacyDeclaration, SingleOwnerAccount};
use starknet_core::types::{BroadcastedInvokeTransaction, InvokeTransactionResult, MaybePendingTransactionReceipt, TransactionReceipt};
use starknet_core::types::contract::{CompiledClass, SierraClass};
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider, ProviderError};
use starknet_signers::LocalWallet;
use crate::bridge::helpers::deploy_utils::RpcAccount;
use crate::utils::constants::{FEE_TOKEN_ADDRESS, MAX_FEE_OVERRIDE};
use crate::utils::utils::wait_for_transaction;

pub struct U256 {
    pub high: FieldElement,
    pub low: FieldElement,
}

pub type TransactionExecution<'a> = Execution<'a, RpcAccount<'a>>;
type TransactionLegacyDeclaration<'a> = LegacyDeclaration<'a, RpcAccount<'a>>;
type TransactionDeclaration<'a> = Declaration<'a, RpcAccount<'a>>;

#[async_trait]
pub trait AccountActions {
    fn transfer_tokens_u256(
        &self,
        recipient: FieldElement,
        transfer_amount: U256,
        nonce: Option<u64>,
    ) -> TransactionExecution;

    fn transfer_tokens(
        &self,
        recipient: FieldElement,
        transfer_amount: FieldElement,
        nonce: Option<u64>,
    ) -> TransactionExecution;

    fn invoke_contract(
        &self,
        address: FieldElement,
        method: &str,
        calldata: Vec<FieldElement>,
        nonce: Option<u64>,
    ) -> TransactionExecution;

    fn declare_contract(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> (TransactionDeclaration, FieldElement, FieldElement);

    fn declare_legacy_contract(&self, path_to_compiled_contract: &str) -> (TransactionLegacyDeclaration, FieldElement, LegacyContractClass);
    fn declare_contract_params_sierra(&self,path_to_sierra: &str, path_to_casm: &str) -> (FieldElement, SierraClass);
    fn declare_contract_params_legacy(&self, path_to_compiled_contract: &str) -> LegacyContractClass;

    async fn prepare_invoke(
        &self,
        calls: Vec<Call>,
        nonce: FieldElement,
        max_fee: FieldElement,
        query_only: bool,
    ) -> BroadcastedInvokeTransaction
        where
            Self: Account + ConnectedAccount,
    {
        let prepared_execution = Execution::new(calls, self).nonce(nonce).max_fee(max_fee).prepared().unwrap();
        prepared_execution.get_invoke_request(query_only).await.unwrap()
    }
}

impl AccountActions for SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet> {
    fn transfer_tokens_u256(
        &self,
        recipient: FieldElement,
        transfer_amount: U256,
        nonce: Option<u64>,
    ) -> TransactionExecution {
        let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
        self.invoke_contract(
            fee_token_address,
            "transfer",
            vec![recipient, transfer_amount.low, transfer_amount.high],
            nonce,
        )
    }

    fn transfer_tokens(
        &self,
        recipient: FieldElement,
        transfer_amount: FieldElement,
        nonce: Option<u64>,
    ) -> TransactionExecution {
        self.transfer_tokens_u256(recipient, U256 { high: FieldElement::ZERO, low: transfer_amount }, nonce)
    }

    fn invoke_contract(
        &self,
        address: FieldElement,
        method: &str,
        calldata: Vec<FieldElement>,
        nonce: Option<u64>,
    ) -> TransactionExecution {
        let calls = vec![Call { to: address, selector: get_selector_from_name(method).unwrap(), calldata }];

        let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

        match nonce {
            Some(nonce) => self.execute(calls).max_fee(max_fee).nonce(nonce.into()),
            None => self.execute(calls).max_fee(max_fee),
        }
    }

    fn declare_contract(
        &self,
        path_to_sierra: &str,
        path_to_casm: &str,
    ) -> (TransactionDeclaration, FieldElement, FieldElement) {
        let sierra: SierraClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_sierra).unwrap(),
        )
            .unwrap();
        let casm: CompiledClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_casm).unwrap(),
        )
            .unwrap();
        let compiled_class_hash = casm.class_hash().unwrap();
        (
            self.declare(sierra.clone().flatten().unwrap().into(), compiled_class_hash)
                // starknet-rs calls estimateFee with incorrect version which throws an error
                .max_fee(FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap()),
            sierra.class_hash().unwrap(),
            compiled_class_hash,
        )
    }

    fn declare_legacy_contract(&self, path_to_compiled_contract: &str) -> (TransactionLegacyDeclaration, FieldElement, LegacyContractClass) {
        let contract_artifact: LegacyContractClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_compiled_contract).unwrap(),
        )
            .unwrap();

        (
            self.declare_legacy(Arc::new(contract_artifact.clone()))
                // starknet-rs calls estimateFee with incorrect version which throws an error
                .max_fee(FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap()),
            contract_artifact.class_hash().unwrap(),
            contract_artifact
        )
    }

    fn declare_contract_params_sierra(&self,path_to_sierra: &str, path_to_casm: &str) -> (FieldElement, SierraClass) {
        let sierra: SierraClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_sierra).unwrap(),
        )
            .unwrap();
        let casm: CompiledClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_casm).unwrap(),
        )
            .unwrap();

        (
            casm.class_hash().unwrap(),
            sierra
        )
    }

    fn declare_contract_params_legacy(&self, path_to_compiled_contract: &str) -> LegacyContractClass {
        let contract_artifact: LegacyContractClass = serde_json::from_reader(
            std::fs::File::open(env!("CARGO_MANIFEST_DIR").to_owned() + "/" + path_to_compiled_contract).unwrap(),
        ).unwrap();

        contract_artifact
    }
}

pub async fn assert_poll<F, Fut>(f: F, polling_time_ms: u64, max_poll_count: u32)
    where
        F: Fn() -> Fut,
        Fut: Future<Output = bool>,
{
    for _poll_count in 0..max_poll_count {
        if f().await {
            return; // The provided function returned true, exit safely.
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(polling_time_ms)).await;
    }

    panic!("Max poll count exceeded.");
}

type TransactionReceiptResult = Result<MaybePendingTransactionReceipt, ProviderError>;

pub async fn get_transaction_receipt(
    rpc: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> TransactionReceiptResult {
    // there is a delay between the transaction being available at the client
    // and the sealing of the block, hence sleeping for 100ms
    assert_poll(|| async { rpc.get_transaction_receipt(transaction_hash).await.is_ok() }, 100, 20).await;

    rpc.get_transaction_receipt(transaction_hash).await
}

pub async fn get_contract_address_from_deploy_tx(
    rpc: &JsonRpcClient<HttpTransport>,
    tx: &InvokeTransactionResult,
) -> Result<FieldElement, ProviderError> {
    let deploy_tx_hash = tx.transaction_hash;

    wait_for_transaction(rpc, deploy_tx_hash).await.unwrap();

    let deploy_tx_receipt = get_transaction_receipt(rpc, deploy_tx_hash).await?;

    let contract_address = assert_matches!(
        deploy_tx_receipt,
        MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(receipt)) => receipt.events.iter().find(|e| e.keys[0] == get_selector_from_name("ContractDeployed").unwrap()).unwrap().data[0]
    );
    Ok(contract_address)
}