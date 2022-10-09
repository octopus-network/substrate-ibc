#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub const LOG_TARGET: &str = "runtime::pallet-ics20-transfer";

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use ibc::applications::transfer::msgs::transfer::MsgTransfer;
	use ibc::events::IbcEvent;
	use ibc::handler::{HandlerOutput, HandlerOutputBuilder};
	use pallet_ibc::Any;
	use pallet_ibc::module::applications::transfer::transfer_handle_callback::TransferModule;
	use pallet_ibc::module::core::ics24_host::{Height, Packet};
	use crate::store_send_packet;
	use crate::LOG_TARGET;


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_ibc::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}


	#[pallet::storage]
	/// (height, port_id, channel_id, sequence) => send-packet event
	pub type SendPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Send packet event
		SendPacket { height: Height, packet: Packet },
		// unsupported event
		UnsupportedEvent,
	}

	impl<T: Config> From<IbcEvent> for Event<T> {
		fn from(event: IbcEvent) -> Self {
			match event {
				IbcEvent::SendPacket(value) => {
					Event::<T>::SendPacket { height: value.height.into(), packet: value.packet.into() }
				},
				_ => Event::<T>::UnsupportedEvent,
			}
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// Parser Msg Transfer Error
		ParserMsgTransferError,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// ICS20 fungible token transfer.
		/// Handling transfer request as sending chain or receiving chain.
		///
		/// Parameters:
		/// - `messages`: A serialized protocol buffer message containing the transfer request.
		///
		/// The relevant events are emitted when successful.
		#[pallet::weight(0)]
		pub fn raw_transfer(
			origin: OriginFor<T>,
			messages: Vec<Any>,
		) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;
			let mut ctx = TransferModule(PhantomData::<T>);

			let messages: Vec<ibc_proto::google::protobuf::Any> = messages
				.into_iter()
				.map(|message| ibc_proto::google::protobuf::Any {
					type_url: String::from_utf8(message.type_url.clone()).unwrap(),
					value: message.value,
				})
				.collect();

			log::trace!(
				target: LOG_TARGET,
				"raw_transfer : {:?} ",
				messages.iter().map(|message| message.type_url.clone()).collect::<Vec<_>>()
			);

			for message in messages {
				let mut handle_out = HandlerOutputBuilder::new();
				let msg_transfer = MsgTransfer::try_from(message)
					.map_err(|_| Error::<T>::ParserMsgTransferError)?;
				let result = ibc::applications::transfer::relay::send_transfer::send_transfer(
					&mut ctx,
					&mut handle_out,
					msg_transfer,
				);
				match result {
					Ok(_value) => {
						log::trace!(target: LOG_TARGET, "raw_transfer Successful!");
					},
					Err(error) => {
						log::trace!(target: LOG_TARGET, "raw_transfer Error : {:?} ", error);
					},
				}

				let HandlerOutput::<()> { result:_, log, events } = handle_out.with_result(());

				log::trace!(target: LOG_TARGET, "raw_transfer log : {:?} ", log);

				// deposit events about send packet event and ics20 transfer event
				for event in events {
					match event {
						IbcEvent::SendPacket(ref send_packet) => {
							store_send_packet::<T>(send_packet);
							Self::deposit_event(event.into());
						},
						_ => {
							log::trace!(target: LOG_TARGET, "raw_transfer event : {:?} ", event);
							Self::deposit_event(event.clone().into());
						},
					}
				}
			}

			Ok(().into())
		}
	}
}

fn store_send_packet<T: Config>(send_packet_event: &ibc::core::ics04_channel::events::SendPacket) {
	use tendermint_proto::Protobuf;
	// store key port_id and channel_id
	let port_id = send_packet_event.packet.source_port.as_bytes().to_vec();
	let channel_id = send_packet_event.packet.source_channel.clone().to_string().as_bytes().to_vec();
	// store value packet
	let packet = send_packet_event.packet.encode_vec().unwrap();
	<SendPacketEvent<T>>::insert(
		(port_id, channel_id, u64::from(send_packet_event.packet.sequence)),
		packet,
	);
}