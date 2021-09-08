#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::{Randomness, Currency, ReservableCurrency, ExistenceRequirement}};
	use frame_system::pallet_prelude::*;
    use codec::{Encode, Decode, Codec};
    use sp_io::hashing::blake2_128;
    use sp_runtime::{traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, CheckedAdd, Zero, One, Bounded }};
    use core::convert::TryInto;
    //use std::convert::TryInto;


    #[derive(Encode, Decode)]
    pub struct Kitty(pub [u8;16]);


    pub type BalanceOf<T> =
            <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    //type KittyIndex = u32;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
        //type Balance: Parameter + Member + Default + Copy + MaybeSerializeDeserialize;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash>;
        /// The currency in which fees are paid and contract balances are held.
        type Currency: Currency<Self::AccountId>  + ReservableCurrency<Self::AccountId>;
        type KittyIndex: Parameter + Member + CheckedAdd + Zero + One + Bounded + Codec + MaybeSerializeDeserialize + AtLeast32BitUnsigned + Default + Copy + sp_std::str::FromStr;
        //type KittyCreateReserve: Get<BalanceOf<T>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;


    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

    #[pallet::storage]
    pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;


    #[pallet::storage]
    pub type Offers<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, BalanceOf<T>, OptionQuery>;



	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
        KittyCreate(T::AccountId, T::KittyIndex), 
        KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
        KittyOffer(T::AccountId, BalanceOf<T>, T::KittyIndex),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
        /// Kitties count overflow
        KittiesCountOverflow,
        /// the give accountid is not kitty owner
        NotOwner,
        /// kitty belong to youself
        IsOwner,
        /// Kitty index is invalid
        InvalidKittyIndex,
        /// The parent of Kitty is same
        SameParentIndex,
        /// Kitty is not offered yet 
        NotOffer,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {

        #[pallet::weight(0)]
        pub fn create(origin: OriginFor<T>,  balance: BalanceOf<T>,) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
           
            let dna = Self::random_value(&who);
            
            let kitty_id = Self::new_kitty(&who, dna).ok_or(Error::<T>::KittiesCountOverflow)?;

           
            
            let reserve: u64 = 100;
            T::Currency::reserve(&who, balance)?;

            Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn transfer(origin: OriginFor<T>,
                        new_owner: T::AccountId,
                        kitty_id: T::KittyIndex,
                        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

            Owner::<T>::insert(kitty_id, Some(new_owner.clone()));
            Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn breed(origin: OriginFor<T>,
                        kitty_id_1: T::KittyIndex,
                        kitty_id_2: T::KittyIndex,
                        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

            let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

            let dna_1 = kitty1.0;
            let dna_2 = kitty2.0;

            let selector = Self::random_value(&who);
            let mut new_dna = [0u8; 16];

            for i in 0..dna_1.len() {
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }

            let kitty_id = Self::new_kitty(&who, new_dna).ok_or(Error::<T>::KittiesCountOverflow)?;
            
            Self::deposit_event(Event::KittyCreate(who, kitty_id));

            Ok(().into())
        }

        ///ower offer a price for sale
        #[pallet::weight(0)]
        pub fn offer(origin: OriginFor<T>,
                        balance: BalanceOf<T>,
                        kitty_id: T::KittyIndex,
                        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
            Offers::<T>::insert(kitty_id, balance.clone());
            Self::deposit_event(Event::KittyOffer(who, balance, kitty_id));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn buy(origin: OriginFor<T>,
                        kitty_id: T::KittyIndex,
                        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let owner = Owner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            let balance = Offers::<T>::get(kitty_id).ok_or(Error::<T>::NotOffer)?;

            ensure!(Some(who.clone()) != Some(owner.clone()), Error::<T>::IsOwner);
			
            T::Currency::transfer(&who, &owner, balance, ExistenceRequirement::AllowDeath)?;

            Owner::<T>::insert(kitty_id, Some(who.clone()));
            Offers::<T>::remove(kitty_id);

            Self::deposit_event(Event::KittyTransfer(owner.clone(), who, kitty_id));

            Ok(().into())
        }


	}

	impl<T:Config> Pallet<T> {
        fn random_value(sender:&T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)

        }

        fn new_kitty(sender:&T::AccountId, dna:[u8; 16]) -> Option<T::KittyIndex> {
            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    if id == T::KittyIndex::max_value() {
                        return None;
                    }
                    id
                },
                None => {
                    //0
                    //T::KittyIndex::default()
                    T::KittyIndex::zero()
                }

            };

            Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
            Owner::<T>::insert(kitty_id, Some(sender.clone()));
            //KittiesCount::<T>::put(kitty_id + 1);
            KittiesCount::<T>::put(kitty_id.checked_add(&T::KittyIndex::one()).unwrap());


            Some(kitty_id)

        }


    }
}
