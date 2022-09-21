use crate::*;
extern crate chrono;
// use chrono::prelude::*;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use std::convert::TryFrom;
use std::time::SystemTime;

// we have to define view functions here.
// but maybe I already adjusted and you would add little funtions.

#[near_bindgen]
impl Contract {
    /// views
    pub fn get_supply_staking_informations(&self) -> U64 {
        U64(self.staking_informations.len())
    }

    pub fn get_supply_by_owner_id(&self, account_id: AccountId) -> U64 {
        let by_owner_id = self.by_owner_id.get(&account_id);
        if let Some(by_owner_id) = by_owner_id {
            U64(by_owner_id.len())
        } else {
            U64(0)
        }
    }

    pub fn get_staking_informations_by_owner_id(
        &self,
        account_id: AccountId,
        from_index: U64,
        limit: u64,
    ) -> Vec<StakeInfo> {
        let mut tmp = vec![];
        let by_owner_id = self.by_owner_id.get(&account_id);
        let staking_informations = if let Some(by_owner_id) = by_owner_id {
            by_owner_id
        } else {
            return vec![];
        };
        let keys = staking_informations.as_vector();
        let start = u64::from(from_index);
        let end = min(start + limit, staking_informations.len());
        for i in start..end {
            tmp.push(
                self.staking_informations
                    .get(&keys.get(i).unwrap())
                    .unwrap(),
            );
        }
        tmp
    }

    pub fn get_staking_information(&self, tokenId: TokenId) -> Option<StakeInfo> {
        self.staking_informations.get(&tokenId)
    }

    pub fn get_claim_amount(&self, account_id: AccountId) -> U128 {
        let nft_staked_arrays =
            self.get_staking_informations_by_owner_id(account_id, U64(0), 10000);
        let mut claimed: u128 = 0;
        let now = env::block_timestamp() / 1000000;
        let now_dt: DateTime<Utc> =
            DateTime::from_utc(NaiveDateTime::from_timestamp((now / 1000) as i64, 0), Utc);
        for stk in nft_staked_arrays {
            let claimed_at = stk.claimed_at as i64;
            let claimed_at_date = NaiveDateTime::from_timestamp((claimed_at / 1000) as i64, 0);
            let dt: DateTime<Utc> = DateTime::from_utc(claimed_at_date, Utc);
            let duration = now_dt - dt;
            claimed += u128::try_from(duration.num_hours() * 20 / 24)
                .ok()
                .unwrap()
                .checked_mul(1000000000000000000000000)
                .unwrap();
        }

        U128::from(claimed)
    }
}
