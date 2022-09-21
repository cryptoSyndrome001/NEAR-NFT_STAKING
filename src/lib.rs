use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey,
    CryptoHash, Gas, PanicOnDefault, Promise,
};
use std::cmp::min;
use std::collections::HashMap;

use crate::external::*;
use crate::internal::*;
use crate::staking::*;
use near_sdk::env::STORAGE_PRICE_PER_BYTE;

mod external;
mod internal;
mod nft_callbacks;
mod staking;
mod staking_views;

near_sdk::setup_alloc!();

//I already adjusted this contract and it has basic info for staking.

// TODO check seller supports storage_deposit at ft_token_id they want to post sale in

const GAS_FOR_FT_TRANSFER: Gas = 35_000_000_000_000;
/// greedy max Tgas for resolve_purchase
const GAS_FOR_ROYALTIES: Gas = 215_000_000_000_000;
const GAS_FOR_NFT_TRANSFER: Gas = 45_000_000_000_000;
const BID_HISTORY_LENGTH_DEFAULT: u8 = 1;
const NO_DEPOSIT: Balance = 0;
const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;
static DELIMETER: &str = "||";

pub type TokenId = String;

// TODO: Capital U128
pub type Payout = HashMap<AccountId, U128>;
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub nft_contract_id: AccountId,
    pub staking_informations: UnorderedMap<TokenId, StakeInfo>,
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub anc_token_id: AccountId,
    pub storage_deposits: LookupMap<AccountId, Balance>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    StakingInformation,
    ByOwnerId,
    ByOwnerIdInner { account_id_hash: CryptoHash },
    NftContractId,
    AncTokenId,
    StorageDeposits,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId) -> Self {
        let this = Self {
            owner_id: owner_id.into(),
            nft_contract_id: String::from("x.paras.near"),
            // kokumokongz.near apenearclub.near
            // nft_contract_id: String::new(StorageKey::NftContractId),
            staking_informations: UnorderedMap::new(StorageKey::StakingInformation),
            anc_token_id: String::from("anctokenwallet.near"),
            // kokumokongz_token_wallet.near
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
        };
        this
    }

    /// TODO remove token (should check if sales can complete even if owner stops supporting token type)

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) {
        let storage_account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(env::predecessor_account_id);
        let deposit = env::attached_deposit();
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            STORAGE_PER_SALE
        );
        let mut balance: u128 = self.storage_deposits.get(&storage_account_id).unwrap_or(0);
        balance += deposit;
        self.storage_deposits.insert(&storage_account_id, &balance);
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        let mut amount = self.storage_deposits.remove(&owner_id).unwrap_or(0);
        let sales = self.by_owner_id.get(&owner_id);
        let len = sales.map(|s| s.len()).unwrap_or_default();
        let diff = u128::from(len) * STORAGE_PER_SALE;
        amount -= diff;
        if amount > 0 {
            Promise::new(owner_id.clone()).transfer(amount);
        }
        if diff > 0 {
            self.storage_deposits.insert(&owner_id, &diff);
        }
    }

    /// views

    pub fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(STORAGE_PER_SALE),
            max: None,
        }
    }

    pub fn storage_minimum_balance(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    pub fn storage_balance_of(&self, account_id: ValidAccountId) -> U128 {
        U128(self.storage_deposits.get(account_id.as_ref()).unwrap_or(0))
    }

    /// deprecated

    pub fn storage_paid(&self, account_id: ValidAccountId) -> U128 {
        U128(self.storage_deposits.get(account_id.as_ref()).unwrap_or(0))
    }

    pub fn storage_amount(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }
}
