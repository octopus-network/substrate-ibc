use crate::{Error, mock::*, ClientType, ConsensusState, ClientState};
use frame_support::{assert_ok, assert_noop};
use sp_core::{Blake2Hasher, Hasher};

#[test]
fn create_client_should_work() {
	let identifier1 = Blake2Hasher::hash("appia".as_bytes());
	let identifier2 = Blake2Hasher::hash("flaminia".as_bytes());
	let height = 0;
	let consensus_state = ConsensusState {
		set_id: 0,
		authorities: vec![],
		commitment_root: Blake2Hasher::hash("root".as_bytes()),
	};

	new_test_ext().execute_with(|| {
		assert_ok!(IbcModule::create_client(identifier1, ClientType::GRANDPA, height.clone(), consensus_state.clone()));
		assert_ok!(IbcModule::create_client(identifier2, ClientType::GRANDPA, height.clone(), consensus_state.clone()));
		assert!(IbcModule::create_client(identifier1, ClientType::GRANDPA, height.clone(), consensus_state).is_err());
	});
}