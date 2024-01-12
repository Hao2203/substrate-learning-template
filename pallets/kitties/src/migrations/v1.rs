use frame_support::{migration::storage_key_iter, pallet_prelude::*, storage::*};

use crate::{Config, Kitties, Kitty, KittyId, Pallet};

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct KittyV0(pub [u8; 16]);

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if on_chain_version != 0 {
		return Weight::zero();
	}
	if current_version != 1 {
		return Weight::zero();
	}

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (index, kitty) in
		storage_key_iter::<KittyId, KittyV0, Blake2_128Concat>(module, item).drain()
	{
		let kitty = Kitty { dna: kitty.0, name: *b"noneabcd" };

		Kitties::<T>::insert(index, kitty);
	}
	Weight::zero()
}
