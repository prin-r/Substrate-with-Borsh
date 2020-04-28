#![cfg_attr(not(feature = "std"), no_std)]

use band_bridge;
#[cfg(feature = "std")]
use borsh::BorshDeserialize;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch};
use frame_system::{self as system};
use sp_std::prelude::*;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + band_bridge::Trait {
    // Add other types and constants required to configure this pallet.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as SimplePriceDB {
        BlockIDToPrice get(fn simple_map): map hasher(blake2_128_concat) T::BlockNumber => u64;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        BlockNumber = <T as system::Trait>::BlockNumber,
    {
        SetPriceAtBlock(u64, BlockNumber),
        DecodeFail(AccountId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        BorshDecodeFail,
        /// Value reached maximum and cannot be incremented further
        StorageOverflow,
        ///
        VerificationFail,
    }
}

// Define struct Price
#[cfg(feature = "std")]
#[derive(BorshDeserialize)]
struct Price {
    px: u64,
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
        pub fn set_price(_origin, proof_data: Vec<u8>) -> dispatch::DispatchResult {
            // Call Bridge contract to verify proof
            let res_opt = <band_bridge::Module<T>>::verify_proof(proof_data.clone());

            match res_opt {
                Some(res) => {
                    #[cfg(feature = "std")]
                    let price = Price::try_from_slice(&res).map_err(|_| Error::<T>::BorshDecodeFail)?;

                    // Call the `system` pallet to get the current block number
                    let current_block = <system::Module<T>>::block_number();

                    // Update key-value
                    #[cfg(feature = "std")]
                    <BlockIDToPrice<T>>::insert(
                        &current_block,
                        &price.px
                    );

                    // Here we are raising the SetPriceAtBlock event
                    #[cfg(feature = "std")]
                    Self::deposit_event(RawEvent::SetPriceAtBlock(price.px, current_block));

                    Ok(())
                },
                None => Err(Error::<T>::VerificationFail)?,
            }
        }
    }
}
