use crate::*;

use crate::context::Context;
use log::{error, info, trace, warn};

use ibc::{
	clients::ics10_grandpa::{consensus_state::ConsensusState as GPConsensusState, header::Header},
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState, client_state::AnyClientState,
			context::ClientReader, error::Error as Ics02Error,
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

impl<T: Config> ConnectionReader for Context<T> {
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [connection_end] conn_id:{:?}",conn_id);

		let connections_path = ConnectionsPath(conn_id.clone()).to_string().as_bytes().to_vec();

		if <Connections<T>>::contains_key(&connections_path) {
			let data = <Connections<T>>::get(&connections_path);
			let ret = ConnectionEnd::decode_vec(&*data).map_err(Ics03Error::invalid_decode)?;

			trace!(target:"runtime::pallet-ibc","in connection : [connection_end] >>  connection_end = {:?}", ret);
			Ok(ret)
		} else {
			trace!(target:"runtime::pallet-ibc","in connection : [connection_end] >> read connection end returns None");
			Err(Ics03Error::connection_mismatch(conn_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [client_state] client_id:{:?}",client_id);

		ClientReader::client_state(self, client_id).map_err(Ics03Error::ics02_client)
	}

	fn host_current_height(&self) -> Height {
		trace!(target:"runtime::pallet-ibc","in connection : [host_current_height]");

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();

		<OldHeight<T>>::put(current_height);

		trace!(target:"runtime::pallet-ibc",
			"in connection : [host_current_height] >> Host current height = {:?}",
			Height::new(REVISION_NUMBER, current_height)
		);
		Height::new(REVISION_NUMBER, current_height).unwrap()
	}

	fn host_oldest_height(&self) -> Height {
		trace!(target:"runtime::pallet-ibc","in connection : [host_oldest_height]");

		let height = <OldHeight<T>>::get();

		trace!(target:"runtime::pallet-ibc",
			"in connection : [host_oldest_height] >> Host oldest height = {:?}",
			Height::new(0, height)
		);
		Height::new(REVISION_NUMBER, height).unwrap()
	}

	fn commitment_prefix(&self) -> CommitmentPrefix {
		trace!(target:"runtime::pallet-ibc","in connection : [commitment_prefix]");

		"ibc".as_bytes().to_vec().try_into().unwrap_or_default()
	}

	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [client_consensus_state] client_id:{:?},heigh:{:?}",client_id,height);

		let ret = ClientReader::consensus_state(self, client_id, height)
			.map_err(Ics03Error::ics02_client);

		if let Ok(value) = ret {
			Ok(value)
		} else {
			// TODO(davirain) template deatil with
			Ok(AnyConsensusState::Grandpa(
				ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
			))
		}
	}

	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [host_consensus_state] _height:{:?}",_height);
		// let result = AnyConsensusState::Grandpa(GPConsensusState::from(Header::default()));

		// trace!(target:"runtime::pallet-ibc","in connection : [host_consensus_state] >>
		// any_consensus_state = {:?}", result); Ok(result)
		// get local chain timestamp
		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));

		let ts = ts.unwrap().into_tm_time().unwrap();
		log::trace!(target:"runtime::pallet-ibc","in connection : [host_timestamp] >> host_timestamp = {:?}", ts);

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u32 = block_number.parse().unwrap_or_default();

		trace!(target:"runtime::pallet-ibc",
			"in connection: [host_height] >> host_height = {:?}",current_height

		);

		//TODO: need to build a real consensus state from substrate chain

		let cs = ibc::clients::ics10_grandpa::consensus_state::ConsensusState {
			commitment: Commitment::default(),
			state_root: CommitmentRoot::from(vec![1, 2, 3]),
			timestamp: ts,
		};
		trace!(target:"runtime::pallet-ibc","in connection : [host_consensus_state] consensus_state = {:?}", cs);
		Ok(AnyConsensusState::Grandpa(cs))
	}

	fn connection_counter(&self) -> Result<u64, Ics03Error> {
		trace!(target:"runtime::pallet-ibc",
			"in connection : [connection_counter]"
		);

		Ok(<ConnectionCounter<T>>::get())
	}
}

impl<T: Config> ConnectionKeeper for Context<T> {
	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [store_connection] connection_id:{:?}, connection_end:{:?}",connection_id,connection_end);

		let connections_path = ConnectionsPath(connection_id).to_string().as_bytes().to_vec();
		let data = connection_end.encode_vec().map_err(Ics03Error::invalid_encode)?;

		// store connection end
		<Connections<T>>::insert(connections_path, data);

		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [store_connection_to_client] connection_id:{:?}, client_id:{:?}",connection_id,client_id);

		let client_connection_paths =
			ClientConnectionsPath(client_id.clone()).to_string().as_bytes().to_vec();

		<ConnectionClient<T>>::insert(client_connection_paths, connection_id.as_bytes().to_vec());
		Ok(())
	}

	fn increase_connection_counter(&mut self) {
		trace!(target:"runtime::pallet-ibc","in connection : [increase_connection_counter]");

		let ret = <ConnectionCounter<T>>::try_mutate(|val| -> Result<(), Ics03Error> {
			let new = val
				.checked_add(1)
				.ok_or_else(Ics03Error::invalid_increment_connection_counter)?;
			*val = new;
			Ok(())
		});
	}
}
