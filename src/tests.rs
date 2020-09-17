use crate::{Error, mock::*, ClientType, ConsensusState};
use frame_support::{assert_ok, assert_noop};
use sp_core::{Blake2Hasher, Hasher};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		// assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		// assert_eq!(TemplateModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			TemplateModule::cause_error(Origin::signed(1)),
			Error::<Test>::NoneValue
		);
	});
}

#[test]
fn test_create_ibc_client() {
//constructor is not visible here due to private fields
/*	let Pair(pair) = AccountKeyring::Bob.pair();
	println!("{:?}", pair);*/
	let set_id = 0;
	let authorities = [].to_vec();
	let commitment_root = Blake2Hasher::hash("commitment_root".as_bytes());
	let identifier = Blake2Hasher::hash("appia".as_bytes());
	let height = 0;

	let consensus_state = ConsensusState {
		set_id,
		authorities,
		commitment_root,
	};

	new_test_ext().execute_with(|| {
		let _result = TemplateModule::create_client(identifier, ClientType::GRANDPA, height, consensus_state);
		let consensus_state = TemplateModule::getConsensusState(identifier.clone(), height.clone());
		assert_eq!(consensus_state.set_id, set_id);
	});
}