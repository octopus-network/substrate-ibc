use crate::*;
use alloc::string::ToString;
use core::str::FromStr;
use log::{error, info, trace, warn};

use crate::context::Context;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState,
			client_state::AnyClientState,
			client_type::ClientType,
			context::{ClientKeeper, ClientReader},
			error::Error as Ics02Error,
		},
		ics23_commitment::commitment::CommitmentRoot,
		ics24_host::identifier::ClientId,
	},
	timestamp::Timestamp,
	Height,
};

impl<T: Config> ClientReader for Context<T> {
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client : [client_type]");

		if <Clients<T>>::contains_key(client_id.as_bytes()) {
			let data = <Clients<T>>::get(client_id.as_bytes());
			let mut data: &[u8] = &data;
			let data = Vec::<u8>::decode(&mut data).map_err(Ics02Error::invalid_codec_decode)?;
			let data = String::from_utf8(data).map_err(Ics02Error::invalid_from_utf8)?;
			let client_type = ClientType::from_str(&data)
				.map_err(|e| Ics02Error::unknown_client_type(e.to_string()))?;
			Ok(client_type)
		} else {
			trace!(target:"runtime::pallet-ibc","in client : [client_type] >> read client_type is None");
			Err(Ics02Error::client_not_found(client_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client : [client_state]");

		if <ClientStates<T>>::contains_key(client_id.as_bytes()) {
			let data = <ClientStates<T>>::get(client_id.as_bytes());
			let result = AnyClientState::decode_vec(&*data).map_err(Ics02Error::invalid_decode)?;
			trace!(target:"runtime::pallet-ibc","in client : [client_state] >> any client_state: {:?}", result);

			Ok(result)
		} else {
			trace!(target:"runtime::pallet-ibc","in client : [client_state] >> read any client state is None");
			Err(Ics02Error::client_not_found(client_id.clone()))
		}
	}

	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, Ics02Error> {
		trace!(target:"runtime::pallet-ibc",
			"in client : [consensus_state]"
		);

		let mut values = <ConsensusStates<T>>::get(client_id.as_bytes());
		values.sort_by(|(height_left, _), (height_right, _)| {
			let height_left = Height::decode(&mut &height_left[..]).unwrap_or_default();
			let height_right = Height::decode(&mut &height_right[..]).unwrap_or_default();
			height_left.cmp(&height_right)
		});

		for item in values.iter() {
			let item_height =
				Height::decode(&mut &item.0.clone()[..]).map_err(Ics02Error::invalid_decode)?;

			if item_height == height {
				let any_consensus_state =
					AnyConsensusState::decode_vec(&*item.1).map_err(Ics02Error::invalid_decode)?;
				trace!(target:"runtime::pallet-ibc",
					"in client : [consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(any_consensus_state);
			}
		}

		Err(Ics02Error::consensus_state_not_found(client_id.clone(), height))
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, Ics02Error> {
		trace!(target:"runtime::pallet-ibc",
			"in client : [next_consensus_state]"
		);

		let mut values = <ConsensusStates<T>>::get(client_id.as_bytes());
		values.sort_by(|(height_left, _), (height_right, _)| {
			let height_left = Height::decode(&mut &height_left[..]).unwrap_or_default();
			let height_right = Height::decode(&mut &height_right[..]).unwrap_or_default();
			height_left.cmp(&height_right)
		});

		for item in values.iter() {
			let item_height =
				Height::decode(&mut &item.0.clone()[..]).map_err(Ics02Error::invalid_decode)?;

			if item_height < height {
				let any_consensus_state =
					AnyConsensusState::decode_vec(&*item.1).map_err(Ics02Error::invalid_decode)?;
				trace!(target:"runtime::pallet-ibc",
					"in client : [consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(Some(any_consensus_state));
			}
		}

		// Ok(Some(AnyConsensusState::Grandpa(
		// 	ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		// )))
		Ok(None)
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, Ics02Error> {
		trace!(target:"runtime::pallet-ibc",
			"in client : [pre_consensus_state]"
		);

		let mut values = <ConsensusStates<T>>::get(client_id.as_bytes());
		values.sort_by(|(height_left, _), (height_right, _)| {
			let height_left = Height::decode(&mut &height_left[..]).unwrap_or_default();
			let height_right = Height::decode(&mut &height_right[..]).unwrap_or_default();
			height_left.cmp(&height_right)
		});

		for item in values.iter() {
			let item_height =
				Height::decode(&mut &item.0.clone()[..]).map_err(Ics02Error::invalid_decode)?;

			if item_height > height {
				let any_consensus_state =
					AnyConsensusState::decode_vec(&*item.1).map_err(Ics02Error::invalid_decode)?;
				trace!(target:"runtime::pallet-ibc",
					"in client : [consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(Some(any_consensus_state));
			}
		}

		// Ok(Some(AnyConsensusState::Grandpa(
		// 	ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		// )))
		Ok(None)
	}

	fn host_height(&self) -> Height {
		trace!(target:"runtime::pallet-ibc","in client : [host_height]");
		let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
		let current_height: u64 = block_number.parse().unwrap_or_default();

		trace!(target:"runtime::pallet-ibc",
			"in channel: [host_height] >> host_height = {:?}",
			Height::new(0, current_height)
		);
		Height::new(0, current_height)
	}

	fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client : [consensus_state]");

		// get local chain timestamp
		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));

		let ts = ts.unwrap().into_tm_time().unwrap();

		log::trace!(target:"runtime::pallet-ibc","in client: [host_timestamp] >> host_timestamp = {:?}", ts);

		//TODO: need to build a real consensus state from substrate chain
		let cs = ibc::clients::ics10_grandpa::consensus_state::ConsensusState {
			parent_hash: vec![0; 10],
			block_number: 0,
			state_root: vec![0; 10],
			extrinsics_root: vec![0; 10],
			digest: vec![0; 10],
			root: CommitmentRoot::from(vec![1, 2, 3]),
			timestamp: ts,
		};
		Ok(AnyConsensusState::Grandpa(cs))
		// Ok(AnyConsensusState::Grandpa(
		// 	ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		// ))
	}

	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client: [pending_host_consensus_state]");
		// get local chain timestamp
		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));
		let ts = ts.unwrap().into_tm_time().unwrap();
		log::trace!(target:"runtime::pallet-ibc","in client: [host_timestamp] >> host_timestamp = {:?}", ts);

		//TODO: need to build a real consensus state from substrate chain
		let cs = ibc::clients::ics10_grandpa::consensus_state::ConsensusState {
			parent_hash: vec![0; 10],
			block_number: 0,
			state_root: vec![0; 10],
			extrinsics_root: vec![0; 10],
			digest: vec![0; 10],
			root: CommitmentRoot::from(vec![1, 2, 3]),
			timestamp: ts,
		};
		Ok(AnyConsensusState::Grandpa(cs))

		// Ok(AnyConsensusState::Grandpa(
		// 	ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		// ))
	}

	fn client_counter(&self) -> Result<u64, Ics02Error> {
		trace!(target:"runtime::pallet-ibc",
			"in client : [client_counter]"
		);

		Ok(<ClientCounter<T>>::get())
	}
}

impl<T: Config> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), Ics02Error> {
		info!("in client : [store_client_type]");

		let client_id = client_id.as_bytes().to_vec();
		let client_type = client_type.as_str().encode();
		<Clients<T>>::insert(client_id, client_type);
		Ok(())
	}

	fn increase_client_counter(&mut self) {
		info!("in client : [increase_client_counter]");

		let ret = <ClientCounter<T>>::try_mutate(|val| -> Result<(), Ics02Error> {
			let new = val.checked_add(1).ok_or_else(Ics02Error::invalid_increase_client_counter)?;
			*val = new;
			Ok(())
		});
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: AnyClientState,
	) -> Result<(), Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client : [store_client_state]");

		let data = client_state.encode_vec().map_err(Ics02Error::invalid_encode)?;
		// store client states key-value
		<ClientStates<T>>::insert(client_id.as_bytes().to_vec(), data);

		// store client states keys
		let ret = <ClientStatesKeys<T>>::try_mutate(|val| -> Result<(), Ics02Error> {
			if let Some(_value) = val.iter().find(|&x| x == client_id.as_bytes()) {
			} else {
				val.push(client_id.as_bytes().to_vec());
			}
			Ok(())
		});

		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: AnyConsensusState,
	) -> Result<(), Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client : [store_consensus_state]");

		let height = height.encode_vec().map_err(Ics02Error::invalid_encode)?;
		let data = consensus_state.encode_vec().map_err(Ics02Error::invalid_encode)?;

		let ct = self.client_type(&client_id.clone())?;
		match ct {
			ClientType::Grandpa => {
				if !<ConsensusStates<T>>::contains_key(client_id.as_bytes()) {
					trace!(target:"runtime::pallet-ibc","in client : [store_consensus_state] need to insert new grandpa onsensus_state !");
					<ConsensusStates<T>>::insert(client_id.as_bytes(), vec![(height, data)]);
				}
			},
			ClientType::Tendermint => {
				trace!(target:"runtime::pallet-ibc","in client : [store_consensus_state] need to insert new tendermint onsensus_state !");
				<ConsensusStates<T>>::insert(client_id.as_bytes(), vec![(height, data)]);
			},
			_ => {
				unimplemented!();
			},
		}

		Ok(())
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client: [store_update_time]");

		let encode_timestamp = serde_json::to_string(&timestamp)
			.map_err(Ics02Error::invalid_serde_json_encode)?
			.as_bytes()
			.to_vec();
		<ClientProcessedTimes<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().map_err(Ics02Error::invalid_encode)?,
			encode_timestamp,
		);

		Ok(())
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), Ics02Error> {
		trace!(target:"runtime::pallet-ibc","in client: [store_update_height]");

		<ClientProcessedHeights<T>>::insert(
			client_id.as_bytes(),
			height.encode_vec().map_err(Ics02Error::invalid_encode)?,
			host_height.encode_vec().map_err(Ics02Error::invalid_encode)?,
		);

		Ok(())
	}
}
