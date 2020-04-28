#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use borsh::{BorshDeserialize, BorshSerialize};

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch};
use frame_system::{self as system, ensure_signed};
use sp_io::crypto::secp256k1_ecdsa_recover;
use sp_io::hashing::keccak_256;

use sp_std::prelude::*;

// use sp_io::crypto::secp256k1_ecdsa_recover;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    // Add other types and constants required to configure this pallet.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[cfg(feature = "std")]
#[derive(Clone, Eq, PartialEq, Default, BorshSerialize, BorshDeserialize)]
pub struct VerifyOracleDataResult {
    oracle_script_id: u64,
    request_time: u64,
    aggregation_time: u64,
    requested_validators_count: u64,
    sufficient_validator_count: u64,
    reported_validators_count: u64,
    params: Vec<u8>,
    data: Vec<u8>,
}

#[cfg(feature = "std")]
#[derive(Clone, Eq, PartialEq, Default, BorshDeserialize)]
pub struct BandProof {
    signature: Vec<u8>,
    result: VerifyOracleDataResult,
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as BridgeStorage {
        Proof get(fn proof): Option<Vec<u8>>;
        pub Validator get(fn validator): Option<Vec<u8>>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        SetValidator(Vec<u8>, AccountId),
        EmitProofData(Vec<u8>, Vec<u8>, Vec<u8>),
        EmitCannotGetFromProofData(),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        NoneValue,
        /// Value reached maximum and cannot be incremented further
        StorageOverflow,
        /// VerificationFail
        VerificationFail,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing errors
        // this includes information about your errors in the node's metadata.
        // it is needed only if you are using errors in your pallet
        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        #[weight = frame_support::weights::SimpleDispatchInfo::default()]
        pub fn test_verify_proof(_origin, proof:Vec<u8>) -> dispatch::DispatchResult {
            let res_opt = <Module<T>>::test_signature_verification(proof);

            match res_opt {
                Some((data,recover_validator,state_validator)) => Self::deposit_event(RawEvent::EmitProofData(data,recover_validator,state_validator)),
                None => Self::deposit_event(RawEvent::EmitCannotGetFromProofData()),
            };

            Ok(())
        }

        #[weight = frame_support::weights::SimpleDispatchInfo::default()]
        pub fn set_validator(origin, validator:Vec<u8>) -> dispatch::DispatchResult {
            let user = ensure_signed(origin)?;

            Validator::put(&validator);

            Self::deposit_event(RawEvent::SetValidator(validator, user));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn verify_proof(band_proof: Vec<u8>) -> Option<Vec<u8>> {
        if band_proof.len() < 8 {
            None
        } else {
            Some((&band_proof[(band_proof.len() - 8)..]).to_vec())
        }
    }

    pub fn test_signature_verification(band_proof: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        #[cfg(feature = "std")]
        return match BandProof::try_from_slice(&band_proof) {
            Ok(bp) => {
                let res = bp.result.try_to_vec();
                match res {
                    Ok(result_bytes) => {
                        let hash_of_result = keccak_256(&result_bytes);
                        let mut sig: [u8; 65] = [0; 65];
                        sig.copy_from_slice(&bp.signature);

                        match secp256k1_ecdsa_recover(&sig, &hash_of_result) {
                            Ok(pubkey) => {
                                let hash_of_pubkey = keccak_256(&pubkey);
                                let v = Validator::get()?;
                                Some((bp.result.data, (&hash_of_pubkey[12..]).to_vec(), v.to_vec()))
                            }
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        };

        None
    }
}
