use crate::Config;
use alloc::string::ToString;
use codec::{Decode, Encode};
use ibc::{
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::{ChannelError, PacketError},
			handler::ModuleExtras,
			msgs::acknowledgement::Acknowledgement,
			packet::Packet,
			Version,
		},
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::context::{Module, ModuleOutputBuilder},
	},
	signer::Signer,
};

use sp_std::marker::PhantomData;

#[derive(Debug, Encode, Decode)]
pub struct IbcTransferModule<T>(pub PhantomData<T>);

impl<T: Config> Module for IbcTransferModule<T> {
	fn on_chan_open_init(
		&mut self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
	) -> Result<(ModuleExtras, Version), ChannelError> {
		ibc::applications::transfer::context::on_chan_open_init(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
		)
		.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_try(
		&mut self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		counterparty_version: &Version,
	) -> Result<(ModuleExtras, Version), ChannelError> {
		ibc::applications::transfer::context::on_chan_open_try(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			counterparty_version,
		)
		.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_ack(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty_version: &Version,
	) -> Result<ModuleExtras, ChannelError> {
		ibc::applications::transfer::context::on_chan_open_ack(
			self,
			port_id,
			channel_id,
			counterparty_version,
		)
		.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_confirm(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		ibc::applications::transfer::context::on_chan_open_confirm(self, port_id, channel_id)
			.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_close_init(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_chan_close_confirm(
		&mut self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		ibc::applications::transfer::context::on_chan_close_confirm(self, port_id, channel_id)
			.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_recv_packet(
		&mut self,
		output: &mut ModuleOutputBuilder,
		packet: &Packet,
		relayer: &Signer,
	) -> Acknowledgement {
		ibc::applications::transfer::context::on_recv_packet(self, output, packet, relayer)
	}

	fn on_acknowledgement_packet(
		&mut self,
		output: &mut ModuleOutputBuilder,
		packet: &Packet,
		acknowledgement: &Acknowledgement,
		relayer: &Signer,
	) -> Result<(), PacketError> {
		ibc::applications::transfer::context::on_acknowledgement_packet(
			self,
			output,
			packet,
			acknowledgement,
			relayer,
		)
		.map_err(|e| PacketError::AppModule { description: e.to_string() })
	}

	fn on_timeout_packet(
		&mut self,
		output: &mut ModuleOutputBuilder,
		packet: &Packet,
		relayer: &Signer,
	) -> Result<(), PacketError> {
		ibc::applications::transfer::context::on_timeout_packet(self, output, packet, relayer)
			.map_err(|e| PacketError::AppModule { description: e.to_string() })
	}
}
