use std::sync::Mutex;

use crate::{block_context::Base, state::DictStateReader};
use blockifier::{
    abi::abi_utils::get_storage_var_address,
    block_context::BlockContext,
    state::{
        cached_state::CachedState,
        state_api::{State, StateReader},
    },
    transaction::{account_transaction::AccountTransaction, transactions::ExecutableTransaction},
};
use starknet_api::{
    core::{calculate_contract_address, ClassHash, ContractAddress, Nonce},
    hash::StarkFelt,
    stark_felt,
    transaction::{
        Calldata, ContractAddressSalt, DeployAccountTransaction, Fee, TransactionHash,
        TransactionSignature, TransactionVersion,
    },
};

pub struct KatanaSequencer {
    pub block_context: BlockContext,
    pub state: Mutex<CachedState<DictStateReader>>,
}

impl KatanaSequencer {
    pub fn new() -> Self {
        Self {
            block_context: BlockContext::base(),
            state: Mutex::new(CachedState::new(DictStateReader::new())),
        }
    }

    pub fn drip_and_deploy_account(
        &self,
        class_hash: ClassHash,
        version: TransactionVersion,
        contract_address_salt: ContractAddressSalt,
        constructor_calldata: Calldata,
        signature: TransactionSignature,
        balance: u64,
    ) -> anyhow::Result<(TransactionHash, ContractAddress)> {
        let contract_address = calculate_contract_address(
            contract_address_salt,
            class_hash,
            &constructor_calldata,
            ContractAddress::default(),
        )
        .unwrap();

        let deployed_account_balance_key =
            get_storage_var_address("ERC20_balances", &[*contract_address.0.key()]).unwrap();
        self.state.lock().unwrap().set_storage_at(
            self.block_context.fee_token_address,
            deployed_account_balance_key,
            stark_felt!(Fee(u128::from(balance)).0 as u64),
        );

        self.deploy_account(
            class_hash,
            version,
            contract_address_salt,
            constructor_calldata,
            signature,
        )
    }

    pub fn deploy_account(
        &self,
        class_hash: ClassHash,
        version: TransactionVersion,
        contract_address_salt: ContractAddressSalt,
        constructor_calldata: Calldata,
        signature: TransactionSignature,
    ) -> anyhow::Result<(TransactionHash, ContractAddress)> {
        let contract_address = calculate_contract_address(
            contract_address_salt,
            class_hash,
            &constructor_calldata,
            ContractAddress::default(),
        )
        .unwrap();

        let account_balance_key =
            get_storage_var_address("ERC20_balances", &[*contract_address.0.key()]).unwrap();
        let max_fee = self
            .state
            .lock()
            .unwrap()
            .get_storage_at(self.block_context.fee_token_address, account_balance_key)?;

        let max_fee_b16 = &max_fee.bytes()[..16];
        // TODO: Compute txn hash
        let tx_hash = TransactionHash::default();
        let tx = AccountTransaction::DeployAccount(DeployAccountTransaction {
            max_fee: Fee(u128::from_be_bytes(max_fee_b16.try_into().unwrap())),
            version,
            class_hash,
            contract_address,
            contract_address_salt,
            constructor_calldata,
            nonce: Nonce(stark_felt!(0)),
            signature,
            transaction_hash: tx_hash,
        });
        tx.execute(&mut self.state.lock().unwrap(), &self.block_context)?;

        Ok((tx_hash, contract_address))
    }
}

impl Default for KatanaSequencer {
    fn default() -> Self {
        Self::new()
    }
}
