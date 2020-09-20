use crate::{Error, mock::*, ClientType, ConsensusState};
use frame_support::{assert_ok, assert_err, dispatch};
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

#[test]
fn bind_port_should_work() {
	let identifier = "port_name".as_bytes().to_vec();
	let module_index = 45 as u8;
	new_test_ext().execute_with(|| {
		assert_ok!(IbcModule::bind_port(identifier.clone(), module_index));
		assert_err!(IbcModule::bind_port(identifier.clone(), module_index), dispatch::DispatchError::Other("Port identifier already exists"));
	});
}