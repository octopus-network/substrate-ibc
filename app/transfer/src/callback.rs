use crate::Config;
use alloc::string::ToString;
use ibc::{
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::{ChannelError, PacketError},
			packet::{Acknowledgement, Packet},
			Version,
		},
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		router::{Module, ModuleExtras},
	},
	Signer,
};
use sp_std::marker::PhantomData;

#[derive(Debug)]
pub struct IbcTransferModule<T> {
	pub ibc_core_context: pallet_ibc::context::Context<T>,
	pub phatom_data: PhantomData<T>,
}

impl<T: Config> IbcTransferModule<T> {
	pub fn new(ibc_core_context: pallet_ibc::context::Context<T>) -> Self {
		Self { ibc_core_context, phatom_data: PhantomData::default() }
	}
}

impl<T: Config> Module for IbcTransferModule<T> {
	fn on_chan_open_init_validate(
		&self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
	) -> Result<Version, ChannelError> {
		ibc::applications::transfer::context::on_chan_open_init_validate(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
		)
		.map_err(|e| ChannelError::AppModule { description: e.to_string() })?;

		Ok(version.clone())
	}

	fn on_chan_open_init_execute(
		&mut self,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_ort_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty: &Counterparty,
		version: &Version,
	) -> Result<(ModuleExtras, Version), ChannelError> {
		Ok((ModuleExtras::empty(), version.clone()))
	}

	fn on_chan_open_try_validate(
		&self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		counterparty_version: &Version,
	) -> Result<Version, ChannelError> {
		ibc::applications::transfer::context::on_chan_open_try_validate(
			self,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			counterparty_version,
		)
		.map_err(|e| ChannelError::AppModule { description: e.to_string() })?;

		Ok(counterparty_version.clone())
	}

	fn on_chan_open_try_execute(
		&mut self,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty: &Counterparty,
		counterparty_version: &Version,
	) -> Result<(ModuleExtras, Version), ChannelError> {
		Ok((ModuleExtras::empty(), counterparty_version.clone()))
	}

	fn on_chan_open_ack_validate(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty_version: &Version,
	) -> Result<(), ChannelError> {
		ibc::applications::transfer::context::on_chan_open_ack_validate(
			self,
			port_id,
			channel_id,
			counterparty_version,
		)
		.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_ack_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty_version: &Version,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_chan_open_confirm_validate(
		&self,
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<(), ChannelError> {
		ibc::applications::transfer::context::on_chan_open_confirm_validate(
			self, port_id, channel_id,
		)
		.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	}

	fn on_chan_open_confirm_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_chan_close_init_validate(
		&self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), ChannelError> {
		Ok(())
	}

	fn on_chan_close_init_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_chan_close_confirm_validate(
		&self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), ChannelError> {
		Ok(())
	}

	fn on_chan_close_confirm_execute(
		&mut self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<ModuleExtras, ChannelError> {
		Ok(ModuleExtras::empty())
	}

	fn on_recv_packet_execute(
		&mut self,
		packet: &Packet,
		_relayer: &Signer,
	) -> (ModuleExtras, Acknowledgement) {
		ibc::applications::transfer::context::on_recv_packet_execute(self, packet)
	}

	fn on_acknowledgement_packet_validate(
		&self,
		packet: &Packet,
		acknowledgement: &Acknowledgement,
		relayer: &Signer,
	) -> Result<(), PacketError> {
		ibc::applications::transfer::context::on_acknowledgement_packet_validate(
			self,
			packet,
			acknowledgement,
			relayer,
		)
		.map_err(|e| PacketError::AppModule { description: e.to_string() })
	}

	fn on_acknowledgement_packet_execute(
		&mut self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
		_relayer: &Signer,
	) -> (ModuleExtras, Result<(), PacketError>) {
		(ModuleExtras::empty(), Ok(()))
	}

	fn on_timeout_packet_validate(
		&self,
		packet: &Packet,
		relayer: &Signer,
	) -> Result<(), PacketError> {
		ibc::applications::transfer::context::on_timeout_packet_validate(self, packet, relayer)
			.map_err(|e| PacketError::AppModule { description: e.to_string() })
	}

	fn on_timeout_packet_execute(
		&mut self,
		_packet: &Packet,
		_relayer: &Signer,
	) -> (ModuleExtras, Result<(), PacketError>) {
		(ModuleExtras::empty(), Ok(()))
	}
}
