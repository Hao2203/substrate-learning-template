#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_io::hashing::blake2_128,
		traits::{Currency, ExistenceRequirement, Randomness},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::AccountIdConversion;

	pub type KittyId = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constent]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBred { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { from: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
		KittyOnSale { who: T::AccountId, kitty_id: KittyId },
		KittyBought { who: T::AccountId, kitty_id: KittyId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		AlreadyOnSale,
		AlreadyOwned,
		NotOnSale,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty(Self::random_value(&who));

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?;
			T::Currency::transfer(
				&who,
				&Self::account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			<Kitties<T>>::insert(kitty_id, &kitty);
			<KittyOwner<T>>::insert(kitty_id, &who);

			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?;
			let kitty_1 = Kitties::<T>::get(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_2 = Kitties::<T>::get(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let selected = Self::random_value(&who);
			let mut dna = [0u8; 16];
			for i in 0..dna.len() {
				dna[i] = (kitty_1.0[i] & selected[i]) | (kitty_2.0[i] & !selected[i]);
			}
			let kitty = Kitty(dna);

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?;
			T::Currency::transfer(
				&who,
				&Self::account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			<Kitties<T>>::insert(kitty_id, &kitty);
			<KittyOwner<T>>::insert(kitty_id, &who);
			<KittyParents<T>>::insert(kitty_id, (kitty_id_1, kitty_id_2));

			Self::deposit_event(Event::KittyBred { who, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn transfer(
			origin: OriginFor<T>,
			recipent: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == who, Error::<T>::NotOwner);

			<KittyOwner<T>>::insert(kitty_id, &recipent);
			Self::deposit_event(Event::KittyTransferred {
				from: who,
				recipient: recipent,
				kitty_id,
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({0})]
		pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(Self::kitty_on_sale(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			KittyOnSale::<T>::insert(kitty_id, ());
			Self::deposit_event(Event::KittyOnSale { who, kitty_id });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight({0})]
		pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			let owner = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner != who, Error::<T>::AlreadyOnSale);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale);

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?;
			// T::Currency::unreserve(&Self::kitty_owner(kitty_id).unwrap(), price);
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;

			KittyOwner::<T>::insert(kitty_id, &who);
			KittyOnSale::<T>::remove(kitty_id);
			Self::deposit_event(Event::KittyBought { who, kitty_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|id| -> Result<KittyId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(1).ok_or(Error::<T>::InvalidKittyId)?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
