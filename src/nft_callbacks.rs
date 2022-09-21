use crate::*;
use near_sdk::PromiseOrValue;
/// approval callbacks from NFT Contracts

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingArgs {
    pub staking_status: String,
}

trait NonFungibleTokenReceiver {
    /// Returns `true` if the token should be returned back to the sender.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool>;
}

#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    /// where we add the sale because we know nft owner can only call nft_transfer

    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        // enforce cross contract call and owner_id is signer

        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();
        assert_ne!(
            nft_contract_id, signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        assert_eq!(
            &previous_owner_id,
            &signer_id,
            "previous_owner_id should be signer_id"
        );

        // // enforce signer's storage is enough to cover + 1 more sale

        // let storage_amount = self.storage_amount().0;
        // let owner_paid_storage = self.storage_deposits.get(&signer_id).unwrap_or(0);
        // let signer_storage_required = (self.get_supply_by_previous_owner_id(signer_id).0 + 1) as u128 * storage_amount;
        // assert!(
        //     owner_paid_storage >= signer_storage_required,
        //     "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
        //     owner_paid_storage, signer_storage_required / STORAGE_PER_SALE, STORAGE_PER_SALE
        // );

        let StakingArgs { staking_status } =
            near_sdk::serde_json::from_str(&msg).expect("Not valid StakingArgs");

        self.staking_informations.insert(
            &token_id,
            &StakeInfo {
                owner_id: previous_owner_id.clone().into(),
                token_id: token_id.clone(),
                created_at: env::block_timestamp() / 1000000,
                claimed_at: env::block_timestamp() / 1000000,
            },
        );

        // extra for views

        let mut by_owner_id = self.by_owner_id.get(&previous_owner_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::ByOwnerIdInner {
                    account_id_hash: hash_account_id(&previous_owner_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        by_owner_id.insert(&token_id);
        self.by_owner_id.insert(&previous_owner_id, &by_owner_id);
        PromiseOrValue::Value(false)
    }
}
