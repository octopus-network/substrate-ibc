use crate::*;

use ibc::{
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::Error as Ics04Error,
			msgs::acknowledgement::Acknowledgement as GenericAcknowledgement,
			packet::Packet as IbcPacket,
			Version,
		},
		ics24_host::identifier::{ChannelId as IbcChannelId, ConnectionId, PortId},
		ics26_routing::context::{Module, ModuleOutputBuilder, OnRecvPacketAck},
	},
	signer::Signer,
};

#[derive(Debug, Default)]
pub struct IBCTransferModule;

impl Module for IBCTransferModule {
	fn on_chan_open_init(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &IbcChannelId,
		_counterparty: &Counterparty,
		_version: &Version,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_chan_open_try(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &IbcChannelId,
		_counterparty: &Counterparty,
		_version: &Version,
		_counterparty_version: &Version,
	) -> Result<Version, Ics04Error> {
		Ok(Version::ics20())
	}

	fn on_chan_open_ack(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &IbcChannelId,
		_counterparty_version: &Version,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_chan_open_confirm(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_chan_close_init(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_chan_close_confirm(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &IbcChannelId,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_recv_packet(
		&self,
		_output: &mut ModuleOutputBuilder,
		_packet: &IbcPacket,
		_relayer: &Signer,
	) -> OnRecvPacketAck {
		OnRecvPacketAck::Nil(Box::new(|_| Ok(())))
	}

	fn on_acknowledgement_packet(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_packet: &IbcPacket,
		_acknowledgement: &GenericAcknowledgement,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		Ok(())
	}

	fn on_timeout_packet(
		&mut self,
		_output: &mut ModuleOutputBuilder,
		_packet: &IbcPacket,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		Ok(())
	}
}
