use crate::*;

use crate::context::Context;
use log::{error, info, trace, warn};

use ibc::{
	clients::ics10_grandpa::{consensus_state::ConsensusState as GPConsensusState, header::Header},
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState, client_state::AnyClientState,
			error::Error as Ics02Error,
		},
		ics03_connection::{
			connection::ConnectionEnd,
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as Ics03Error,
		},
		ics23_commitment::commitment::{CommitmentPrefix, CommitmentRoot},
		ics24_host::identifier::{ClientId, ConnectionId},
	},
	timestamp::Timestamp,
	Height,
};

impl<T: Config> ConnectionReader for Context<T> {
	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [connection_end] conn_id:{:?}",conn_id);

		if <Connections<T>>::contains_key(conn_id.as_bytes()) {
			let data = <Connections<T>>::get(conn_id.as_bytes());
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

		// ClientReader::client_state(self, client_id)
		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			let result = AnyClientState::decode_vec(&*data).map_err(Ics03Error::invalid_decode)?;
			trace!(target:"runtime::pallet-ibc","in connection : [client_state] >> client_state: {:?}", result);

			Ok(result)
		} else {
			trace!(target:"runtime::pallet-ibc","in connection : [client_state] >> read client_state is None");
			Err(Ics03Error::frozen_client(client_id.clone()))
		}
	}

	fn host_current_height(&self) -> Height {
		trace!(target:"runtime::pallet-ibc","in connection : [host_current_height]");

		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();

		<OldHeight<T>>::put(current_height);

		trace!(target:"runtime::pallet-ibc",
			"in connection : [host_current_height] >> Host current height = {:?}",
			Height::new(0, current_height)
		);
		Height::new(0, current_height)
	}

	fn host_oldest_height(&self) -> Height {
		trace!(target:"runtime::pallet-ibc","in connection : [host_oldest_height]");

		let height = <OldHeight<T>>::get();

		trace!(target:"runtime::pallet-ibc",
			"in connection : [host_oldest_height] >> Host oldest height = {:?}",
			Height::new(0, height)
		);
		Height::new(0, height)
	}

	fn connection_counter(&self) -> Result<u64, Ics03Error> {
		trace!(target:"runtime::pallet-ibc",
			"in connection : [connection_counter]"
		);

		Ok(<ConnectionCounter<T>>::get())
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

		// ClientReader::consensus_state(self, client_id, height)
		let encode_height = height.clone().encode_vec().map_err(Ics03Error::invalid_encode)?;
		let value = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in value.iter() {
			if item.0 == encode_height {
				let any_consensus_state =
					AnyConsensusState::decode_vec(&*item.1).map_err(Ics03Error::invalid_decode)?;
				trace!(target:"runtime::pallet-ibc",
					"in connection : [client_consensus_state] consensus_state: {:?}",any_consensus_state
				);

				return Ok(any_consensus_state)
			}
		}

		Err(Ics03Error::missing_consensus_height())
		// Ok(AnyConsensusState::Grandpa(
		// 	ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		// ))
		// Ok()
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
}

impl<T: Config> ConnectionKeeper for Context<T> {
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

	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [store_connection] connection_id:{:?}, connection_end:{:?}",connection_id,connection_end);

		let data = connection_end.encode_vec().map_err(Ics03Error::invalid_encode)?;

		// store connection end
		<Connections<T>>::insert(connection_id.as_bytes().to_vec(), data);

		// store connection id vector for rpc
		let ret = <ConnectionsKeys<T>>::try_mutate(|val| -> Result<(), Ics03Error> {
			if let Some(_value) = val.iter().find(|&x| x == connection_id.as_bytes()) {
			} else {
				val.push(connection_id.as_bytes().to_vec());
			}
			Ok(())
		});

		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), Ics03Error> {
		trace!(target:"runtime::pallet-ibc","in connection : [store_connection_to_client] connection_id:{:?}, client_id:{:?}",connection_id,client_id);

		<ConnectionClient<T>>::insert(
			client_id.as_bytes().to_vec(),
			connection_id.as_bytes().to_vec(),
		);
		Ok(())
	}
}
