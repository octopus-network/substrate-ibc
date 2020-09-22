use crate::{Error, mock::*, ClientType, ConsensusState};
use frame_support::{assert_ok, assert_err, dispatch};
use sp_core::{Blake2Hasher, Hasher, H256};

fn create_client_func() {
	let identifier1 = Blake2Hasher::hash("appia".as_bytes());
	let identifier2 = Blake2Hasher::hash("flaminia".as_bytes());
	let height = 0;
	let consensus_state = ConsensusState {
		set_id: 0,
		authorities: vec![],
		commitment_root: Blake2Hasher::hash("root".as_bytes()),
	};

	assert_ok!(IbcModule::create_client(identifier1, ClientType::GRANDPA, height.clone(), consensus_state.clone()));
	assert_ok!(IbcModule::create_client(identifier2, ClientType::GRANDPA, height.clone(), consensus_state.clone()));
	assert_err!(IbcModule::create_client(identifier1, ClientType::GRANDPA, height.clone(), consensus_state), Error::<Test>::ClientIdExist);
}

#[test]
fn create_client_should_work() {
	new_test_ext().execute_with(|| {
		create_client_func()
	});
}

fn bind_port_func() {
	let identifier = "bank".as_bytes().to_vec();
	let module_index = 45 as u8;
	assert_ok!(IbcModule::bind_port(identifier.clone(), module_index));
	assert_err!(IbcModule::bind_port(identifier.clone(), module_index), Error::<Test>::PortIdBinded);
}

#[test]
fn bind_port_should_work() {
	new_test_ext().execute_with(|| {
		bind_port_func();
	});
}

fn conn_open_init_func() {
	let identifier = Blake2Hasher::hash("appia-connection".as_bytes());
	let desired_counterparty_connection_identifier = Blake2Hasher::hash("flaminia-connection".as_bytes());
	let client_identifier = hex::decode("53a954d6a7b1c595e025226e5f2a1782fdea30cd8b0d207ed4cdb040af3bfa10").unwrap();
	let client_identifier = H256::from_slice(&client_identifier);
	let counterparty_client_identifier = hex::decode("779ca65108d1d515c3e4bc2e9f6d2f90e27b33b147864d1cd422d9f92ce08e03").unwrap();
	let counterparty_client_identifier = H256::from_slice(&counterparty_client_identifier);

	assert_err!(IbcModule::conn_open_init(identifier, desired_counterparty_connection_identifier, client_identifier, counterparty_client_identifier), Error::<Test>::ClientIdNotExist);

	let identifier1 = Blake2Hasher::hash("appia".as_bytes());
	let height = 0;
	let consensus_state = ConsensusState {
		set_id: 0,
		authorities: vec![],
		commitment_root: Blake2Hasher::hash("root".as_bytes()),
	};
	IbcModule::create_client(identifier1, ClientType::GRANDPA, height, consensus_state);

	assert_ok!(IbcModule::conn_open_init(identifier, desired_counterparty_connection_identifier, client_identifier, counterparty_client_identifier));
	assert_err!(IbcModule::conn_open_init(identifier, desired_counterparty_connection_identifier, client_identifier, counterparty_client_identifier), Error::<Test>::ConnectionIdExist);
}

#[test]
fn conn_open_init_should_work() {
	new_test_ext().execute_with(|| {
		conn_open_init_func();
	});
}
