use crate::*;

use crate::utils::host_height;
use ibc::{
	applications::transfer::{acknowledgement::Acknowledgement, error::Error as Ics20Error},
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			context::ChannelKeeper,
			error::Error as Ics04Error,
			msgs::acknowledgement::Acknowledgement as GenericAcknowledgement,
			packet::{Packet as IbcPacket, PacketResult},
			Version,
		},
		ics24_host::identifier::{ChannelId as IbcChannelId, ConnectionId, PortId},
		ics26_routing::context::{Module, ModuleOutputBuilder, OnRecvPacketAck},
	},
	events::IbcEvent,
	signer::Signer,
};

#[derive(Debug)]
pub struct TransferModule<T: Config>(pub PhantomData<T>);

impl<T: Config> Module for TransferModule<T> {
	fn on_chan_open_init(
		&mut self,
		output: &mut ModuleOutputBuilder,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &IbcChannelId,
		counterparty: &Counterparty,
		version: &Version,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_chan_open_init(
			&mut ctx,
			output,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_chan_open_try(
		&mut self,
		output: &mut ModuleOutputBuilder,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &IbcChannelId,
		counterparty: &Counterparty,
		version: &Version,
		counterparty_version: &Version,
	) -> Result<Version, Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_chan_open_try(
			&mut ctx,
			output,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
			counterparty_version,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_chan_open_ack(
		&mut self,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &IbcChannelId,
		counterparty_version: &Version,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_chan_open_ack(
			&mut ctx,
			output,
			port_id,
			channel_id,
			counterparty_version,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_chan_open_confirm(
		&mut self,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_chan_open_confirm(
			&mut ctx, output, port_id, channel_id,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_chan_close_init(
		&mut self,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_chan_close_init(
			&mut ctx, output, port_id, channel_id,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_chan_close_confirm(
		&mut self,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_chan_close_confirm(
			&mut ctx, output, port_id, channel_id,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_recv_packet(
		&self,
		output: &mut ModuleOutputBuilder,
		packet: &IbcPacket,
		relayer: &Signer,
	) -> OnRecvPacketAck {
		let mut ctx = Context::<T>::new();

		let on_recv_packet_ack =
			ibc::applications::transfer::context::on_recv_packet(&mut ctx, output, packet, relayer);

		match on_recv_packet_ack {
			OnRecvPacketAck::Nil(write_fn) => OnRecvPacketAck::Nil(write_fn),
			OnRecvPacketAck::Successful(ack, write_fn) => {
				let ack = ack
					.as_any()
					.downcast_ref::<Acknowledgement>()
					.expect("downcast_ref GenericAcknowledgement error");

				let result = ibc::core::ics04_channel::handler::write_acknowledgement::process(
					&mut ctx,
					packet.clone(),
					ack.as_ref().to_vec(),
				);
				match result {
					Ok(packet_result) => {
						if let PacketResult::WriteAck(write_ack) = packet_result.result {
							let _ = ctx.store_packet_acknowledgement(
								(write_ack.port_id.clone(), write_ack.channel_id, write_ack.seq),
								write_ack.ack_commitment,
							);
						}

						// Emit write acknowledgement event
						let host_current_height = host_height::<T>();
						Pallet::<T>::deposit_event(
							vec![IbcEvent::WriteAcknowledgement(
								ibc::core::ics04_channel::events::WriteAcknowledgement {
									height: Height::new(REVISION_NUMBER, host_current_height)
										.into(),
									packet: packet.clone().into(),
									ack: ack.as_ref().to_vec(),
								}
							)].into()
						);

						// write ack acknowledgement
						if let IbcEvent::WriteAcknowledgement(write_ack_event) =
							packet_result.events.first().unwrap()
						{
							store_write_ack::<T>(write_ack_event);
						}

						OnRecvPacketAck::Successful(Box::new(Acknowledgement::success()), write_fn)
					},
					Err(error) => OnRecvPacketAck::Successful(
						Box::new(Acknowledgement::from_error(Ics20Error::ics04_channel(error))),
						write_fn,
					),
				}
			},
			OnRecvPacketAck::Failed(ack) => OnRecvPacketAck::Failed(ack),
		}
	}

	fn on_acknowledgement_packet(
		&mut self,
		output: &mut ModuleOutputBuilder,
		packet: &IbcPacket,
		acknowledgement: &GenericAcknowledgement,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_acknowledgement_packet(
			&mut ctx,
			output,
			packet,
			acknowledgement,
			relayer,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_timeout_packet(
		&mut self,
		output: &mut ModuleOutputBuilder,
		packet: &IbcPacket,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::new();

		ibc::applications::transfer::context::on_timeout_packet(&mut ctx, output, packet, relayer)
			.map_err(Ics04Error::ics20_transfer)
	}
}
