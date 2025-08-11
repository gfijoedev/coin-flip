use hex::encode;
use near_sdk::{
    env::{self},
    ext_contract,
    json_types::U128,
    near, require, AccountId, Gas, NearToken, Promise, PromiseError,
};
use omni_transaction::signer::types::SignatureResponse;

const CALLBACK_GAS: Gas = Gas::from_tgas(5);

mod chain_signature;

#[allow(dead_code)]
#[ext_contract(my_contract)]
trait MyContract {
    fn mpc_callback(&self, account_id: AccountId);
}

#[near(contract_state)]
pub struct Contract {
    flips: u128,
}

impl Default for Contract {
    fn default() -> Self {
        Self { flips: 0 }
    }
}

#[near]
impl Contract {
    pub fn get_flips(&self) -> U128 {
        U128(self.flips)
    }

    pub fn flip(&mut self) -> Promise {
        // let deposit = env::attached_deposit();
        // require!(
        //     deposit > NearToken::from_millinear(100),
        //     "Deposit must be greater than 0.1 NEAR"
        // );
        // env::log_str(&format!("Deposited {}", deposit));

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
    ) -> String {
        match call_result {
            Ok(signature_response) => {
                // get bytes from signature
                let r_bytes = hex::decode(signature_response.big_r.affine_point)
                    .expect("failed to decode affine_point to bytes");
                let s_bytes = hex::decode(signature_response.s.scalar)
                    .expect("failed to decode scalar to bytes");

                // hash bytes to randomness
                let rng: String = encode(env::sha256(&[r_bytes, s_bytes].concat()));

                rng
            }
            Err(error) => {
                env::log_str(&format!("mpc callback failed with error: {:?}", error));
                "".to_owned()
            }
        }
    }
}
