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

		ibc::applications::transfer::context::on_chan_open_init(
			self,
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

		ibc::applications::transfer::context::on_chan_open_try(
			self,
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

		ibc::applications::transfer::context::on_chan_open_ack(
			self,
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

		ibc::applications::transfer::context::on_chan_open_confirm(
			self, output, port_id, channel_id,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_chan_close_init(
		&mut self,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {

		ibc::applications::transfer::context::on_chan_close_init(
			self, output, port_id, channel_id,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_chan_close_confirm(
		&mut self,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {

		ibc::applications::transfer::context::on_chan_close_confirm(
			self, output, port_id, channel_id,
		)
		.map_err(Ics04Error::ics20_transfer)
	}

	fn on_recv_packet(
		&self,
		output: &mut ModuleOutputBuilder,
		packet: &IbcPacket,
		relayer: &Signer,
	) -> OnRecvPacketAck {

		ibc::applications::transfer::context::on_recv_packet(self, output, packet, relayer)
	}

	fn on_acknowledgement_packet(
		&mut self,
		output: &mut ModuleOutputBuilder,
		packet: &IbcPacket,
		acknowledgement: &GenericAcknowledgement,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {

		ibc::applications::transfer::context::on_acknowledgement_packet(
			self,
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

		ibc::applications::transfer::context::on_timeout_packet(self, output, packet, relayer)
			.map_err(Ics04Error::ics20_transfer)
	}
}
