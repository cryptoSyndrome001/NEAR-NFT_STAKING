use crate::*;
use near_sdk::promise_result_as_success;
extern crate chrono;
// use chrono::prelude::*;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use std::convert::TryFrom;
use std::time::SystemTime;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeInfo {
    pub owner_id: AccountId,
    pub token_id: String,
    pub created_at: u64,
    pub claimed_at: u64,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn claim_reward(&mut self) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let claim_amount = self.get_claim_amount(account_id.clone());

        ext_contract::ft_transfer(
            account_id.clone(),
            claim_amount,
            None,
            &self.anc_token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        );

        let by_owner_id = self.by_owner_id.get(&account_id);
        let staking_informations = if let Some(by_owner_id) = by_owner_id {
            by_owner_id
        } else {
            let empty_value = UnorderedSet::new(StorageKey::StakingInformation);
            empty_value
        };
        let keys = staking_informations.to_vec();
        for key in keys {
            let mut stake_info: StakeInfo = self.staking_informations.get(&key).unwrap();
            stake_info.claimed_at = env::block_timestamp() / 1000000;
            self.staking_informations.insert(&key, &stake_info);
        }
    }

    #[payable]
    pub fn unstake(&mut self, token_id: String) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();

        let stake_info: StakeInfo = self.staking_informations.get(&token_id).unwrap();

        assert_eq!(&account_id, &stake_info.owner_id, "Must be owner");
        let mut claimed: u128 = 0;
        let now = env::block_timestamp() / 1000000;
        let now_dt: DateTime<Utc> =
            DateTime::from_utc(NaiveDateTime::from_timestamp((now / 1000) as i64, 0), Utc);
        let claimed_at = stake_info.claimed_at as i64;
        let claimed_at_date = NaiveDateTime::from_timestamp((claimed_at / 1000) as i64, 0);
        let dt: DateTime<Utc> = DateTime::from_utc(claimed_at_date, Utc);
        let duration = now_dt - dt;
        claimed = u128::try_from(duration.num_hours() * 20 / 24)
            .ok()
            .unwrap()
            .checked_mul(1000000000000000000000000)
            .unwrap();

        ext_contract::ft_transfer(
            account_id.clone(),
            U128::from(claimed),
            None,
            &self.anc_token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        );

        ext_contract::nft_transfer(
            account_id.clone(),
            token_id.clone(),
            0,
            "Unstaking".to_string(),
            &self.nft_contract_id,
            1,
            GAS_FOR_NFT_TRANSFER,
        );

        let token_id_value = token_id.clone();
        self.staking_informations.remove(&token_id_value);
        let mut by_owner_id = self.by_owner_id.get(&account_id).unwrap();
        by_owner_id.remove(&token_id);
        self.by_owner_id.insert(&account_id, &by_owner_id);
    }
}
