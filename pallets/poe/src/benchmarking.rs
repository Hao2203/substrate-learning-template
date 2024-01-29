use frame_system::RawOrigin;
use sp_std::prelude::*;

benchmarks! {
	create_claim {
		let caller = whitelisted_caller();
	}: _(RawOrigin::Root, 0, 0),
}
