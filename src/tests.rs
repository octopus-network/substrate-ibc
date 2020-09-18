use crate::{Error, mock::*, ClientType, ConsensusState, ClientState};
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
	let set_id = 0;
	let authorities = vec![];
	let commitment_root = Blake2Hasher::hash("state_root".as_bytes());
	let identifier = Blake2Hasher::hash("appia".as_bytes());
	let height = 0;
	let frozen_height = None;
	let connections = vec![];
	let channels = vec![];

	let consensus_state = ConsensusState {
		set_id,
		authorities: authorities.clone(),
		commitment_root,
	};

	new_test_ext().execute_with(|| {
		let _result = TemplateModule::create_client(identifier, ClientType::GRANDPA, height.clone(), consensus_state);
		let consensus_state = TemplateModule::getConsensusState(identifier.clone(), height.clone());
		assert_eq!(consensus_state.set_id, set_id);
		assert_eq!(consensus_state.commitment_root, commitment_root);
		assert_eq!(consensus_state.authorities, authorities.clone());

		let client_state = TemplateModule::getClientState(&identifier);
		assert_eq!(client_state.latest_height, height);
		assert_eq!(client_state.frozen_height, frozen_height);
		assert_eq!(client_state.connections, connections);
		assert_eq!(client_state.channels, channels);
	});
}