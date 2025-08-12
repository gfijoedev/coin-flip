use hex::encode;
use near_sdk::{
    env::{self},
    ext_contract,
    json_types::U128,
    log, near, require, AccountId, Gas, NearToken, Promise, PromiseError,
};
use omni_transaction::signer::types::SignatureResponse;

mod chain_signature;

const FLIP_COST: NearToken = NearToken::from_millinear(100);
const FLIP_KEEP: NearToken = NearToken::from_millinear(95);
const CALLBACK_GAS: Gas = Gas::from_tgas(5);

#[allow(dead_code)]
#[ext_contract(my_contract)]
trait MyContract {
    fn mpc_callback(&self, account_id: AccountId);
}

#[near(contract_state)]
pub struct Contract {
    flips: u128,
    pool: u128,
    paid: u128,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            flips: 0,
            pool: 0,
            paid: 0,
        }
    }
}

#[near]
impl Contract {
    pub fn stats(&self) -> (U128, U128, U128) {
        (U128(self.flips), U128(self.pool), U128(self.paid))
    }

    #[payable]
    pub fn flip(&mut self) -> Promise {
        let deposit = env::attached_deposit();
        require!(deposit == FLIP_COST, "Deposit must be 0.1 NEAR");

        self.pool = self
            .pool
            .checked_add(FLIP_KEEP.as_yoctonear())
            .expect("pool overflow");
        self.flips += 1;

        let account_id = env::predecessor_account_id();
        let random_seed = env::random_seed_array();

        chain_signature::internal_request_signature(
            account_id.to_string(),
            encode(random_seed),
            "Ecdsa".to_owned(),
        )
        .then(
            my_contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .mpc_callback(account_id),
        )
    }

    #[private]
    pub fn mpc_callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
        account_id: AccountId,
    ) -> bool {
        match call_result {
            Ok(signature_response) => {
                // get bytes from signature
                let r_bytes = hex::decode(signature_response.big_r.affine_point)
                    .expect("failed to decode affine_point to bytes");
                let s_bytes = hex::decode(signature_response.s.scalar)
                    .expect("failed to decode scalar to bytes");

                let hash = env::sha256(&[r_bytes, s_bytes].concat());
                let bytes: &[u8; 16] = unsafe { slice_to_array_unchecked(&hash[0..16]) };
                let rng = u128::from_le_bytes(*bytes);
                let result = rng % 2 == 0;

                log!("flip result: {:?}", result);

                if result {
                    let payout = self.pool;
                    self.pool = 0;
                    self.paid = self.paid.checked_add(payout).expect("paid overflow");
                    Promise::new(account_id).transfer(NearToken::from_yoctonear(payout));
                }

                result
            }
            Err(error) => {
                env::log_str(&format!("mpc callback failed with error: {:?}", error));
                false
            }
        }
    }
}

unsafe fn slice_to_array_unchecked<T, const N: usize>(slice: &[T]) -> &[T; N] {
    debug_assert!(slice.len() == N);
    &*(slice as *const _ as *const _)
}
