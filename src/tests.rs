use super::pallet::ConsensusStates;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_deliver() {
	let alice: AccountId = AccountKeyring::Alice.into();
	let origin = Origin::signed(alice);
	let any: Any = Any {type_url: vec![1,2, 3], value: vec![1,2 3]};
	let tmp: u8 = 0;
	new_test_ext().execute_with(|| {
		assert_ok!(Ibc::deliver(origin, vec![any], tmp));
	})
}
