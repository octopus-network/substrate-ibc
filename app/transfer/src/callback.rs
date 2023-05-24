use crate::Config;
use codec::{Decode, Encode};
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

#[derive(Debug, Encode, Decode)]
pub struct IbcTransferModule<T>(pub PhantomData<T>);

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
		todo!()
	}

	fn on_chan_open_init_execute(
		&mut self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
	) -> Result<(ModuleExtras, Version), ChannelError> {
		todo!()
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
		todo!()
	}

	fn on_chan_open_try_execute(
		&mut self,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		counterparty_version: &Version,
	) -> Result<(ModuleExtras, Version), ChannelError> {
		todo!()
	}

	fn on_chan_open_ack_validate(
		&self,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty_version: &Version,
	) -> Result<(), ChannelError> {
		Ok(())
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
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(), ChannelError> {
		Ok(())
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

	// Note: no `on_recv_packet_validate()`
	// the `onRecvPacket` callback always succeeds
	// if any error occurs, than an "error acknowledgement"
	// must be returned

	fn on_recv_packet_execute(
		&mut self,
		packet: &Packet,
		relayer: &Signer,
	) -> (ModuleExtras, Acknowledgement) {
		todo!()
	}

	fn on_acknowledgement_packet_validate(
		&self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
		_relayer: &Signer,
	) -> Result<(), PacketError> {
		todo!()
	}

	fn on_acknowledgement_packet_execute(
		&mut self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
		_relayer: &Signer,
	) -> (ModuleExtras, Result<(), PacketError>) {
		todo!()
	}

	/// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

	fn on_timeout_packet_validate(
		&self,
		packet: &Packet,
		relayer: &Signer,
	) -> Result<(), PacketError> {
		todo!()
	}

	/// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

	fn on_timeout_packet_execute(
		&mut self,
		packet: &Packet,
		relayer: &Signer,
	) -> (ModuleExtras, Result<(), PacketError>) {
		todo!()
	}
	// fn on_chan_open_init(
	// 	&mut self,
	// 	order: Order,
	// 	connection_hops: &[ConnectionId],
	// 	port_id: &PortId,
	// 	channel_id: &ChannelId,
	// 	counterparty: &Counterparty,
	// 	version: &Version,
	// ) -> Result<(ModuleExtras, Version), ChannelError> {
	// 	ibc::applications::transfer::context::on_chan_open_init(
	// 		self,
	// 		order,
	// 		connection_hops,
	// 		port_id,
	// 		channel_id,
	// 		counterparty,
	// 		version,
	// 	)
	// 	.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	// }

	// fn on_chan_open_try(
	// 	&mut self,
	// 	order: Order,
	// 	connection_hops: &[ConnectionId],
	// 	port_id: &PortId,
	// 	channel_id: &ChannelId,
	// 	counterparty: &Counterparty,
	// 	counterparty_version: &Version,
	// ) -> Result<(ModuleExtras, Version), ChannelError> {
	// 	ibc::applications::transfer::context::on_chan_open_try(
	// 		self,
	// 		order,
	// 		connection_hops,
	// 		port_id,
	// 		channel_id,
	// 		counterparty,
	// 		counterparty_version,
	// 	)
	// 	.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	// }

	// fn on_chan_open_ack(
	// 	&mut self,
	// 	port_id: &PortId,
	// 	channel_id: &ChannelId,
	// 	counterparty_version: &Version,
	// ) -> Result<ModuleExtras, ChannelError> {
	// 	ibc::applications::transfer::context::on_chan_open_ack(
	// 		self,
	// 		port_id,
	// 		channel_id,
	// 		counterparty_version,
	// 	)
	// 	.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	// }

	// fn on_chan_open_confirm(
	// 	&mut self,
	// 	port_id: &PortId,
	// 	channel_id: &ChannelId,
	// ) -> Result<ModuleExtras, ChannelError> {
	// 	ibc::applications::transfer::context::on_chan_open_confirm(self, port_id, channel_id)
	// 		.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	// }

	// fn on_chan_close_init(
	// 	&mut self,
	// 	_port_id: &PortId,
	// 	_channel_id: &ChannelId,
	// ) -> Result<ModuleExtras, ChannelError> {
	// 	Ok(ModuleExtras::empty())
	// }

	// fn on_chan_close_confirm(
	// 	&mut self,
	// 	port_id: &PortId,
	// 	channel_id: &ChannelId,
	// ) -> Result<ModuleExtras, ChannelError> {
	// 	ibc::applications::transfer::context::on_chan_close_confirm(self, port_id, channel_id)
	// 		.map_err(|e| ChannelError::AppModule { description: e.to_string() })
	// }

	// fn on_recv_packet(
	// 	&mut self,
	// 	output: &mut ModuleOutputBuilder,
	// 	packet: &Packet,
	// 	relayer: &Signer,
	// ) -> Acknowledgement {
	// 	ibc::applications::transfer::context::on_recv_packet(self, output, packet, relayer)
	// }

	// fn on_acknowledgement_packet(
	// 	&mut self,
	// 	output: &mut ModuleOutputBuilder,
	// 	packet: &Packet,
	// 	acknowledgement: &Acknowledgement,
	// 	relayer: &Signer,
	// ) -> Result<(), PacketError> {
	// 	ibc::applications::transfer::context::on_acknowledgement_packet(
	// 		self,
	// 		output,
	// 		packet,
	// 		acknowledgement,
	// 		relayer,
	// 	)
	// 	.map_err(|e| PacketError::AppModule { description: e.to_string() })
	// }

	// fn on_timeout_packet(
	// 	&mut self,
	// 	output: &mut ModuleOutputBuilder,
	// 	packet: &Packet,
	// 	relayer: &Signer,
	// ) -> Result<(), PacketError> {
	// 	ibc::applications::transfer::context::on_timeout_packet(self, output, packet, relayer)
	// 		.map_err(|e| PacketError::AppModule { description: e.to_string() })
	// }
}
