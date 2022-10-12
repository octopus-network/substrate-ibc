use crate::*;

use crate::context::Context;
use log::trace;

use ibc::{
	core::{
		ics02_client::{
			client_state::ClientState, consensus_state::ConsensusState, context::ClientReader,
		},
		ics03_connection::{
			connection::ConnectionEnd,
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as Ics03Error,
		},
		ics23_commitment::commitment::{CommitmentPrefix, CommitmentRoot},
		ics24_host::{
			identifier::{ClientId, ConnectionId},
			path::{ClientConnectionsPath, ConnectionsPath},
		},
	},
	timestamp::Timestamp,
	Height,
};
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};

impl<T: Config> ConnectionReader for Context<T> {
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
		let connections_path = ConnectionsPath(conn_id.clone()).to_string().as_bytes().to_vec();

		if <Connections<T>>::contains_key(&connections_path) {
			let data = <Connections<T>>::get(&connections_path);
			let ret = ConnectionEnd::decode_vec(&data)
				.map_err(|_| Ics03Error::implementation_specific())?;
			Ok(ret)
		} else {
			Err(Ics03Error::connection_mismatch(conn_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics03Error> {
		ClientReader::client_state(self, client_id).map_err(Ics03Error::ics02_client)
	}

	fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, Ics03Error> {
		todo!()
	}

	fn host_current_height(&self) -> Height {
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();
		<OldHeight<T>>::put(current_height);
		Height::new(REVISION_NUMBER, current_height).unwrap()
	}

	fn host_oldest_height(&self) -> Height {
		let height = <OldHeight<T>>::get();
		Height::new(REVISION_NUMBER, height).unwrap()
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		"ibc".as_bytes().to_vec().try_into().unwrap_or_default()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Box<dyn ConsensusState>, Ics03Error> {
		let ret = ClientReader::consensus_state(self, client_id, height)
			.map_err(Ics03Error::ics02_client);

		if let Ok(value) = ret {
			Ok(value)
		} else {
			// TODO(davirain) ics20-transfer deatil with
			todo!()
		}
	}

	fn host_consensus_state(&self, _height: Height) -> Result<Box<dyn ConsensusState>, Ics03Error> {
		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));

		let ts = ts.unwrap().into_tm_time().unwrap();
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u32 = block_number.parse().unwrap_or_default();

		//TODO: need to build a real consensus state from substrate chain

		// let cs = ibc::clients::ics10_grandpa::consensus_state::ConsensusState {
		// 	commitment: Commitment::default(),
		// 	state_root: CommitmentRoot::from(vec![1, 2, 3]),
		// 	timestamp: ts,
		// };
		// trace!(target:"runtime::pallet-ibc","in connection : [host_consensus_state]
		// consensus_state = {:?}", cs); Ok(AnyConsensusState::Grandpa(cs))
		todo!()
	}

	fn connection_counter(&self) -> Result<u64, Ics03Error> {
		Ok(<ConnectionCounter<T>>::get())
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), Ics03Error> {
		let connections_path = ConnectionsPath(connection_id).to_string().as_bytes().to_vec();
		let data =
			connection_end.encode_vec().map_err(|_| Ics03Error::implementation_specific())?;
		<Connections<T>>::insert(connections_path, data);

		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), Ics03Error> {
		let client_connection_paths =
			ClientConnectionsPath(client_id.clone()).to_string().as_bytes().to_vec();

		<ConnectionClient<T>>::insert(client_connection_paths, connection_id.as_bytes().to_vec());
		Ok(())
	}

	fn increase_connection_counter(&mut self) {
		let _ = <ConnectionCounter<T>>::try_mutate(|val| -> Result<(), ()> {
			let new = val.checked_add(1).expect("Add Overflow");
			*val = new;
			Ok(())
		});
	}
}
