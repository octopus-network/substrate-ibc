use crate::{utils::LOG_TARGET, *};
use core::{str::FromStr, time::Duration};
use log::{error, trace};

use crate::{context::Context, utils::host_height};
use ibc::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState, client_state::AnyClientState,
			context::ClientReader,
		},
		ics03_connection::{
			connection::ConnectionEnd, context::ConnectionReader, error::Error as ICS03Error,
		},
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{
				AcknowledgementCommitment as IbcAcknowledgementCommitment,
				PacketCommitment as IbcPacketCommitment,
			},
			context::{ChannelKeeper, ChannelReader},
			error::Error as ICS04Error,
			packet::{Receipt, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};

/// A context supplying all the necessary read-only dependencies for
/// processing any `ChannelMsg`.
impl<T: Config> ChannelReader for Context<T> {
	/// Returns the ChannelEnd for the given `port_id` and `channel_id`.
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [channel_end] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();

		let encode_channel_end = <Channels<T>>::get(encode_port_id, encode_channel_id);

		let channel_end = ChannelEnd::decode_vec(&*encode_channel_end).map_err(|_| {
			ICS04Error::channel_not_found(port_channel_id.clone().0, port_channel_id.clone().1)
		})?;

		trace!(target: LOG_TARGET, "in channel : [channel_end] >> channel_end = {:?}", channel_end);
		Ok(channel_end)
	}

	/// Returns the ConnectionState for the given identifier `connection_id`.
	fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [connection_end] >> connection_id = {:?}",
			connection_id
		);

		let encode_connection_id = connection_id.as_bytes().to_vec();
		let encode_connection_end = <Connections<T>>::get(encode_connection_id);

		let connection_end = ConnectionEnd::decode_vec(&*encode_connection_end)
			.map_err(|_| ICS04Error::connection_not_open(connection_id.clone()))?;

		trace!(
			target: LOG_TARGET,
			"In channel : [connection_end] >> connection_end = {:?}",
			connection_end
		);
		Ok(connection_end)
	}

	/// Returns the vector tuple port_id and channel_id for the given
	/// identifier `connection_id`.
	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [connection_channels] >> connection_end = {:?}",
			conn_id
		);

		let encode_connection_id = conn_id.as_bytes();
		if <ChannelsConnection<T>>::contains_key(encode_connection_id) {
			let port_and_channel_id = <ChannelsConnection<T>>::get(encode_connection_id);

			let mut vectors_port_id_and_channel_id = vec![];

			for item in port_and_channel_id.iter() {
				let string_port_id =
					String::from_utf8(item.0.clone()).map_err(ICS04Error::invalid_from_utf8)?;
				let port_id =
					PortId::from_str(string_port_id.as_str()).map_err(ICS04Error::identifier)?;

				let string_channel_id =
					String::from_utf8(item.1.clone()).map_err(ICS04Error::invalid_from_utf8)?;
				let channel_id = ChannelId::from_str(string_channel_id.as_str())
					.map_err(ICS04Error::identifier)?;

				vectors_port_id_and_channel_id.push((port_id, channel_id));
			}

			trace!(
				target: LOG_TARGET,
				"in channel : [connection_channels] >> Vector<(PortId, ChannelId)> =  {:?}",
				vectors_port_id_and_channel_id
			);
			Ok(vectors_port_id_and_channel_id)
		} else {
			Err(ICS04Error::connection_not_open(conn_id.clone()))
		}
	}

	/// Returns the ClientState for the given identifier `client_id`.
	/// Necessary dependency towards proof verification.
	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS04Error> {
		trace!(target: LOG_TARGET, "in channel : [client_state] >> client_id = {:?}", client_id);

		let encode_client_id = client_id.as_bytes();
		let encode_any_client_state = <ClientStates<T>>::get(encode_client_id);

		let any_consensus_state = AnyClientState::decode_vec(&*encode_any_client_state)
			.map_err(|_| ICS04Error::frozen_client(client_id.clone()))?;

		trace!(
			target: LOG_TARGET,
			"in channel : [client_state] >> Any client state: {:?}",
			any_consensus_state
		);
		Ok(any_consensus_state)
	}

	/// Returns the AnyConsensusState for the given
	/// identifier `client_id` and at the specified `height`.
	fn client_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [client_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let encode_height = height.encode_vec().map_err(|_| ICS04Error::invalid_encode())?;
		let encode_any_consensus_state = <ConsensusStates<T>>::get(client_id.as_bytes());

		for item in encode_any_consensus_state.iter() {
			if item.0 == encode_height {
				let any_consensus_state =
					AnyConsensusState::decode_vec(&*item.1).map_err(ICS04Error::invalid_decode)?;
				trace!(
					target: LOG_TARGET,
					"in channel: [client_consensus_state] >> any consensus state = {:?}",
					any_consensus_state
				);
				return Ok(any_consensus_state)
			}
		}
		trace!(
			target: LOG_TARGET,
			"in channel : [client_consensus_state] >> read about client_id consensus_state error"
		);

		// Err(ICS04Error::frozen_client(client_id.clone()))
		Ok(AnyConsensusState::Grandpa(
			ibc::clients::ics10_grandpa::consensus_state::ConsensusState::default(),
		))
	}

	/// Return `next send sequence ` by given identifier `port_id` and `channel_id`.
	fn get_next_sequence_send(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [get_next_sequence] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();

		let encode_sequence = <NextSequenceSend<T>>::get(encode_port_id, encode_channel_id);

		trace!(
			target: LOG_TARGET,
			"in channel : [get_next_sequence] >> sequence  = {:?}",
			encode_sequence
		);
		Ok(Sequence::from(encode_sequence))
	}

	/// Return `next recv sequence ` by given identifier `port_id` and `channel_id`.
	fn get_next_sequence_recv(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [get_next_sequence_recv] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();

		let encode_sequence = <NextSequenceRecv<T>>::get(encode_port_id, encode_channel_id);

		trace!(
			target: LOG_TARGET,
			"in channel : [get_next_sequence_recv] >> sequence = {:?}",
			encode_sequence
		);

		Ok(Sequence::from(encode_sequence))
	}

	/// Return `next ack sequence ` by given identifier `port_id` and `channel_id`.
	fn get_next_sequence_ack(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [get_next_sequence_ack] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();

		let encode_sequence = <NextSequenceAck<T>>::get(encode_port_id, encode_channel_id);

		trace!(
			target: LOG_TARGET,
			"in channel : [get_next_sequence_ack] >> sequence = {}",
			encode_sequence
		);
		Ok(Sequence::from(encode_sequence))
	}

	/// Returns the `PacketCommitment` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_commitment(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<IbcPacketCommitment, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel : [get_packet_commitment] \
			>> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0,
			key.1,
			key.2
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);

		if <PacketCommitment<T>>::contains_key((
			&encode_port_id,
			&encode_channel_id,
			encode_sequence,
		)) {
			let encode_packet_commitment =
				<PacketCommitment<T>>::get((&encode_port_id, &encode_channel_id, encode_sequence));

			let packet_commitment = IbcPacketCommitment::from(encode_packet_commitment);

			trace!(
				target: LOG_TARGET,
				"in channel : [get_packet_commitment] >> packet_commitment = {:?}",
				packet_commitment
			);
			Ok(packet_commitment)
		} else {
			trace!(
				target: LOG_TARGET,
				"in channel : [get_packet_commitment] >> read get packet commitment return None"
			);
			Err(ICS04Error::packet_commitment_not_found(key.2))
		}
	}

	/// Returns the `Receipt` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_receipt(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<Receipt, ICS04Error> {
		trace!(
			target:LOG_TARGET,
			"in channel : [get_packet_receipt] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0, key.1, key.2
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);

		if <PacketReceipt<T>>::contains_key((&encode_port_id, &encode_channel_id, encode_sequence))
		{
			let encode_receipt =
				<PacketReceipt<T>>::get((&encode_port_id, &encode_channel_id, encode_sequence));

			let string_receipt =
				String::from_utf8(encode_receipt).map_err(ICS04Error::invalid_from_utf8)?;

			let receipt = match string_receipt.as_ref() {
				"Ok" => Receipt::Ok,
				_ => unreachable!(),
			};
			trace!(
				target: LOG_TARGET,
				"in channel : [get_packet_receipt] >> packet_receipt = {:?}",
				receipt
			);
			Ok(receipt)
		} else {
			error!(
				target: LOG_TARGET,
				"in channel : [get_packet_receipt] >> read get packet receipt not found"
			);
			Err(ICS04Error::packet_receipt_not_found(key.2))
		}
	}

	/// Returns the `Acknowledgements` for the given identifier `(PortId, ChannelId, Sequence)`.
	fn get_packet_acknowledgement(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<IbcAcknowledgementCommitment, ICS04Error> {
		trace!(
			target:LOG_TARGET,
			"in channel : [get_packet_acknowledgement] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0, key.1, key.2
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);

		if <Acknowledgements<T>>::contains_key((
			&encode_port_id,
			&encode_channel_id,
			encode_sequence,
		)) {
			let encode_acknowledgement =
				<Acknowledgements<T>>::get((&encode_port_id, &encode_channel_id, encode_sequence));

			let acknowledgement = IbcAcknowledgementCommitment::from(encode_acknowledgement);
			trace!(
				target: LOG_TARGET,
				"in channel : [get_packet_acknowledgement] >> packet_acknowledgement = {:?}",
				acknowledgement
			);
			Ok(acknowledgement)
		} else {
			error!(
				target: LOG_TARGET,
				"in channel : [get_packet_acknowledgement] >> get acknowledgement not found"
			);
			Err(ICS04Error::packet_acknowledgement_not_found(key.2))
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: Vec<u8>) -> Vec<u8> {
		trace!(target: LOG_TARGET, "in channel: [hash]");

		sp_io::hashing::sha2_256(&value).to_vec()
	}

	/// Returns the current height of the local chain.
	fn host_height(&self) -> Height {
		trace!(target: LOG_TARGET, "in channel: [host_height]");

		let revision_number = 0; // TODO, in the future fix
		let revision_height = host_height::<T>();

		trace!(
			target: LOG_TARGET,
			"in channel: [host_height] >> host_height = {:?}",
			revision_height
		);
		Height::new(revision_number, revision_height)
	}

	/// Returns the current timestamp of the local chain.
	fn host_timestamp(&self) -> Timestamp {
		trace!(target: LOG_TARGET, "in channel: [host_timestamp]");

		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.map_err(|e| panic!("{:?}, caused by {:?} from pallet timestamp_pallet", e, time));
		trace!(
			target: LOG_TARGET,
			"in channel: [host_timestamp] >> host_timestamp = {:?}",
			ts.unwrap()
		);

		ts.unwrap()
	}

	/// Returns the `ConsensusState` for the host (local) chain at a specific height.
	fn host_consensus_state(&self, height: Height) -> Result<AnyConsensusState, ICS04Error> {
		trace!(target: LOG_TARGET, "in channel: [host_consensus_state] >> height = {:?}", height);

		ConnectionReader::host_consensus_state(self, height).map_err(ICS04Error::ics03_connection)
	}

	/// Returns the pending `ConsensusState` of the host (local) chain.
	fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, ICS04Error> {
		trace!(target: LOG_TARGET, "in channel: [pending_host_consensus_stata]");

		ClientReader::pending_host_consensus_state(self)
			.map_err(|e| ICS04Error::ics03_connection(ICS03Error::ics02_client(e)))
	}

	/// Returns the time when the client state for the given [`client_id`] was
	/// updated with a header for the given [`Height`]
	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Timestamp, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [client_update_time] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let encode_client_id = client_id.as_bytes().to_vec();
		let encode_height = height.encode_vec().map_err(|_| ICS04Error::invalid_encode())?;

		if <ClientUpdateTime<T>>::contains_key(&encode_client_id, &encode_height) {
			let time = <ClientUpdateTime<T>>::get(&encode_client_id, &encode_height);
			// TODO: Need to handle unwrap()
			let timestamp = Timestamp::from_nanoseconds(time).unwrap();
			Ok(timestamp)
		} else {
			error!(target: LOG_TARGET, "in channel: [client_update_time] processed time not found");
			Err(ICS04Error::processed_time_not_found(client_id.clone(), height))
		}
	}

	/// Returns the height when the client state for the given [`ClientId`] was
	/// updated with a header for the given [`Height`]
	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Height, ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [client_update_height] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let encode_client_id = client_id.as_bytes().to_vec();
		let encode_height = height.encode_vec().map_err(|_| ICS04Error::invalid_encode())?;

		if <ClientUpdateHeight<T>>::contains_key(&encode_client_id, &encode_height) {
			let host_height = <ClientUpdateHeight<T>>::get(&encode_client_id, &encode_height);
			let host_height =
				Height::decode(&mut &host_height[..]).map_err(ICS04Error::invalid_decode)?;
			Ok(host_height)
		} else {
			error!(
				target: LOG_TARGET,
				"in channel: [client_update_height] processed height not found"
			);
			Err(ICS04Error::processed_height_not_found(client_id.clone(), height))
		}
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ICS04Error> {
		trace!(target: LOG_TARGET, "in channel: [channel_counter]");

		Ok(<Pallet<T> as Store>::ChannelCounter::get())
	}

	/// Returns the maximum expected tiome per block.
	fn max_expected_time_per_block(&self) -> Duration {
		trace!(target: LOG_TARGET, "in channel: [max_expected_time_per_block]");

		Duration::from_secs(6)
	}
}

impl<T: Config> ChannelKeeper for Context<T> {
	fn store_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		commitment: IbcPacketCommitment,
	) -> Result<(), ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [store_packet_commitment] \
			>> port_id = {:?}, channel_id = {:?}, sequence = {:?}, packet_commitment = {:?}",
			key.0,
			key.1,
			key.2,
			commitment
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);
		let encode_packet_commitment = commitment.into_vec();

		// insert packet commitment key-value
		<PacketCommitment<T>>::insert(
			(encode_port_id.to_vec(), encode_channel_id.to_vec(), encode_sequence),
			encode_packet_commitment,
		);

		// insert packet commitment keys
		<PacketCommitmentKeys<T>>::try_mutate(|val| -> Result<(), ICS04Error> {
			val.push((encode_port_id.to_vec(), encode_channel_id.to_vec(), encode_sequence));
			Ok(())
		})
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [delete_packet_commitment] \
			>> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0,
			key.1,
			key.2
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);

		// delete packet commitment
		<PacketCommitment<T>>::remove((&encode_port_id, &encode_channel_id, encode_sequence));

		// delete packet commitment keys
		<PacketCommitmentKeys<T>>::try_mutate(|val| -> Result<(), ICS04Error> {
			let index = val
				.iter()
				.position(|value| {
					value == &(encode_port_id.clone(), encode_channel_id.clone(), encode_sequence)
				})
				.ok_or_else(ICS04Error::packet_commitment_keys_not_found)?;
			let ret = val.remove(index);
			assert_eq!(ret, (encode_port_id.clone(), encode_channel_id.clone(), encode_sequence));
			Ok(())
		})
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [store_packet_receipt] \
			>> port_id = {:?}, channel_id = {:?}, sequence = {:?}, receipt = {:?}",
			key.0,
			key.1,
			key.2,
			receipt
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);
		let encode_receipt = match receipt {
			Receipt::Ok => "Ok".as_bytes(),
		};

		<PacketReceipt<T>>::insert(
			(&encode_port_id, &encode_channel_id, encode_sequence),
			encode_receipt,
		);

		Ok(())
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack_commitment: IbcAcknowledgementCommitment,
	) -> Result<(), ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [store_packet_acknowledgement] \
			>> port_id = {:?}, channel_id = {:?}, sequence = {:?}, ack_commitment = {:?}",
			key.0,
			key.1,
			key.2,
			ack_commitment
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);

		// store packet acknowledgement key-value
		<Acknowledgements<T>>::insert(
			(&encode_port_id, &encode_channel_id, encode_sequence),
			ack_commitment.into_vec(),
		);

		// store packet acknowledgement keys
		<AcknowledgementsKeys<T>>::try_mutate(|val| -> Result<(), ICS04Error> {
			val.push((encode_port_id, encode_channel_id, encode_sequence));
			Ok(())
		})
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [delete_packet_acknowledgement] \
			>> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			key.0,
			key.1,
			key.2
		);

		let encode_port_id = key.0.as_bytes().to_vec();
		let encode_channel_id = key.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(key.2);

		// remove acknowledgements
		<Acknowledgements<T>>::remove((&encode_port_id, &encode_channel_id, encode_sequence));

		// remove acknowledgement keys for rpc
		<AcknowledgementsKeys<T>>::try_mutate(|val| -> Result<(), ICS04Error> {
			let index = val
				.iter()
				.position(|value| {
					value == &(encode_port_id.clone(), encode_channel_id.clone(), encode_sequence)
				})
				.ok_or_else(ICS04Error::acknowledgements_keys_not_found)?;
			let ret = val.remove(index);
			assert_eq!(&ret, &(encode_port_id.clone(), encode_channel_id.clone(), encode_sequence));
			Ok(())
		})
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), ICS04Error> {
		trace!(
			target:LOG_TARGET,
			"in channel: [store_connection_channels] >> connection_id = {:?}, port_id = {:?}, channel_id = {:?}",
			conn_id, port_channel_id.0, port_channel_id.1
		);

		let encode_connection_id = conn_id.as_bytes().to_vec();
		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();

		let port_and_channel_id = (encode_port_id.to_vec(), encode_channel_id.to_vec());

		if <ChannelsConnection<T>>::contains_key(&encode_connection_id) {
			trace!(
				target: LOG_TARGET,
				"in channel: [store_connection_channels] >> insert port_channel_id"
			);
			// if connection_identifier exist
			<ChannelsConnection<T>>::try_mutate(
				&encode_connection_id,
				|value| -> Result<(), ICS04Error> {
					value.push(port_and_channel_id);
					Ok(())
				},
			)
			.map_err(|_| ICS04Error::invalid_store_channels_connection())
		} else {
			// if connection_identifier no exist
			trace!(
				target: LOG_TARGET,
				"in channel: [store_connection_channels] >> init ChannelsConnection"
			);

			<ChannelsConnection<T>>::insert(&encode_connection_id, vec![port_and_channel_id]);
			Ok(())
		}
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), ICS04Error> {
		trace!(
			target: LOG_TARGET,
			"in channel: [store_channel] >> port_id = {:?}, channel_id = {:?}, channel_end = {:?}",
			port_channel_id.0,
			port_channel_id.1,
			channel_end
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();
		let encode_channel_end =
			channel_end.encode_vec().map_err(|_| ICS04Error::invalid_encode())?;

		// store channels key-value
		<Channels<T>>::insert(&encode_port_id, &encode_channel_id, encode_channel_end);

		// store channels keys for rpc
		<ChannelsKeys<T>>::try_mutate(|val| -> Result<(), ICS04Error> {
			if let Some(_value) =
				val.iter().find(|&x| x == &(encode_port_id.clone(), encode_channel_id.clone()))
			{
			} else {
				val.push((encode_port_id.clone(), encode_channel_id.clone()));
			}

			Ok(())
		})
		.map_err(|_| ICS04Error::invalid_store_channels_keys())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		trace!(
			target:LOG_TARGET,
			"in channel: [store_next_sequence_send] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			port_channel_id.0, port_channel_id.1, seq
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(seq);

		<NextSequenceSend<T>>::insert(&encode_port_id, &encode_channel_id, encode_sequence);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		trace!(
			target:LOG_TARGET,
			"in channel: [store_next_sequence_recv] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			port_channel_id.0, port_channel_id.1, seq
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(seq);

		<NextSequenceRecv<T>>::insert(&encode_port_id, &encode_channel_id, encode_sequence);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		trace!(
			target:LOG_TARGET,
			"in channel: [store_next_sequence_ack] >> port_id = {:?}, channel_id = {:?}, sequence = {:?}",
			port_channel_id.0, port_channel_id.1, seq
		);

		let encode_port_id = port_channel_id.0.as_bytes().to_vec();
		let encode_channel_id = port_channel_id.1.to_string().as_bytes().to_vec();
		let encode_sequence = u64::from(seq);

		<NextSequenceAck<T>>::insert(&encode_port_id, &encode_channel_id, encode_sequence);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		trace!(target: LOG_TARGET, "in channel: [increase_channel_counter]");

		<ChannelCounter<T>>::try_mutate(|val| -> Result<(), ICS04Error> {
			let new = val.checked_add(1).ok_or_else(ICS04Error::ivalid_increase_channel_counter)?;
			*val = new;
			Ok(())
		})
		.expect("increase_channel_counter error");
	}
}
