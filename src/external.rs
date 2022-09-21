use crate::*;

/// external contract calls
/* 
These functions are used to call outer side contract functions like NFT contract and FT contract
so if you want to just transfer nft you can use "nft_transfer"
or if you want to transfer nft with payment you can use "nft_transfer_payout". this will return the required payout amount send to the prev owner and also NFT creators(belong to royalty)
"ft_transfer" is used for transfering FT tokens.
Not for transfer $Near.
*/
#[ext_contract(ext_contract)]
trait ExtContract {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: String,
    );
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>
    );
}