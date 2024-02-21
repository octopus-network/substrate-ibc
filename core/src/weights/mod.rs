mod mock_client_weight;

use super::*;
use crate::{weights::mock_client_weight::MockClientWeightInfo, MOCK_CLIENT_TYPE};
use alloc::boxed::Box;
use core::marker::PhantomData;
use frame_support::pallet_prelude::Weight;
use ibc::core::{
	ics02_client::{
		context::ClientReader,
		msgs::{
			create_client::MsgCreateClient, misbehaviour::MsgSubmitMisbehaviour,
			update_client::MsgUpdateClient, upgrade_client::MsgUpgradeClient, ClientMsg,
		},
	},
	ics03_connection::{
		context::ConnectionReader,
		msgs::{
			conn_open_ack::MsgConnectionOpenAck, conn_open_confirm::MsgConnectionOpenConfirm,
			conn_open_init::MsgConnectionOpenInit, conn_open_try::MsgConnectionOpenTry,
			ConnectionMsg,
		},
	},
	ics04_channel::msgs::{
		acknowledgement::MsgAcknowledgement, chan_close_confirm::MsgChannelCloseConfirm,
		chan_close_init::MsgChannelCloseInit, chan_open_ack::MsgChannelOpenAck,
		chan_open_confirm::MsgChannelOpenConfirm, chan_open_init::MsgChannelOpenInit,
		chan_open_try::MsgChannelOpenTry, recv_packet::MsgRecvPacket, timeout::MsgTimeout,
		timeout_on_close::MsgTimeoutOnClose, ChannelMsg, PacketMsg,
	},
	ics24_host::identifier::{ChannelId, ClientId, PortId},
	ics26_routing::msgs::MsgEnvelope,
};
use pallet_ibc_utils::CallbackWeight;

pub trait WeightInfo<T> {
	fn create_client(msg_create_client: MsgCreateClient) -> Weight;
	fn misbehaviour(msg_misbehaviour: MsgSubmitMisbehaviour) -> Weight;
	fn update_client(msg_update_client: MsgUpdateClient) -> Weight;
	fn upgrade_client(msg_upgrade_client: MsgUpgradeClient) -> Weight;

	fn conn_open_init(msg_conn_open_init: MsgConnectionOpenInit) -> Weight;
	fn conn_try_open(msg_conn_try_open: MsgConnectionOpenTry) -> Weight;
	fn conn_open_ack(msg_conn_open_ack: MsgConnectionOpenAck) -> Weight;
	fn conn_open_confirm(msg_conn_open_confirm: MsgConnectionOpenConfirm) -> Weight;

	fn channel_open_init(msg_channel_open_init: MsgChannelOpenInit) -> Weight;
	fn channel_open_try(msg_channel_open_try: MsgChannelOpenTry) -> Weight;
	fn channel_open_ack(msg_channel_open_ack: MsgChannelOpenAck) -> Weight;
	fn channel_open_confirm(msg_channel_open_confirm: MsgChannelOpenConfirm) -> Weight;
	fn channel_close_init(msg_channel_close_init: MsgChannelCloseInit) -> Weight;
	fn channel_close_confirm(msg_channel_close_confirm: MsgChannelCloseConfirm) -> Weight;

	fn recv_packet(msg_recv_packet: MsgRecvPacket) -> Weight;
	fn ack_packet(msg_ack_packet: MsgAcknowledgement) -> Weight;
	fn timeout_packet(msg_timeout_packet: MsgTimeout) -> Weight;
	fn timeout_on_close_packet(msg_timout_onclose_packet: MsgTimeoutOnClose) -> Weight;
}

impl<T: Config> WeightInfo<T> for ()
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	fn create_client(msg_create_client: MsgCreateClient) -> Weight {
		let context = Context::<T>::new();
		if let Ok(decode_client_state) =
			ClientReader::decode_client_state(&context, msg_create_client.client_state)
		{
			match decode_client_state.client_type().as_str() {
				MOCK_CLIENT_TYPE => {
					let mock_client = MockClientWeightInfo::<T>::new();
					mock_client.create_client_mock()
				},
				_ => Weight::default(),
			}
		} else {
			Weight::default()
		}
	}

	fn misbehaviour(_msg_misbehaviour: MsgSubmitMisbehaviour) -> Weight {
		Weight::default()
	}

	fn update_client(msg_update_client: MsgUpdateClient) -> Weight {
		let client_type = msg_update_client
			.client_id
			.as_str()
			.rsplit_once('-')
			.map(|(client_type_str, ..)| client_type_str);
		match client_type {
			Some(ty) if ty.contains("mock") => {
				let mock_client = MockClientWeightInfo::<T>::new();
				mock_client.update_mock_client()
			},
			_ => Weight::default(),
		}
	}

	fn upgrade_client(msg_upgrade_client: MsgUpgradeClient) -> Weight {
		let context = Context::<T>::new();
		if let Ok(decode_client_state) =
			ClientReader::decode_client_state(&context, msg_upgrade_client.client_state)
		{
			match decode_client_state.client_type().as_str() {
				MOCK_CLIENT_TYPE => {
					let mock_client = MockClientWeightInfo::<T>::new();
					mock_client.upgrade_mock_client()
				},
				_ => Weight::default(),
			}
		} else {
			Weight::default()
		}
	}

	fn conn_open_init(msg_conn_open_init: MsgConnectionOpenInit) -> Weight {
		let client_type = msg_conn_open_init
			.client_id_on_a
			.as_str()
			.rsplit_once('-')
			.map(|(client_type_str, ..)| client_type_str);
		match client_type {
			Some(ty) if ty.contains("mock") => {
				let mock_client = MockClientWeightInfo::<T>::new();
				mock_client.conn_open_init_mock()
			},
			_ => Weight::default(),
		}
	}

	fn conn_try_open(msg_conn_try_open: MsgConnectionOpenTry) -> Weight {
		let client_type = msg_conn_try_open
			.client_id_on_b
			.as_str()
			.rsplit_once('-')
			.map(|(client_type_str, ..)| client_type_str);
		match client_type {
			Some(ty) if ty.contains("mock") => {
				let mock_client = MockClientWeightInfo::<T>::new();
				mock_client.conn_try_open_mock()
			},
			_ => Weight::default(),
		}
	}

	fn conn_open_ack(_msg_conn_open_ack: MsgConnectionOpenAck) -> Weight {
		let connection_id = _msg_conn_open_ack.conn_id_on_a;
		let ctx = Context::<T>::new();
		let connection_end = ctx.connection_end(&connection_id).unwrap_or_default();
		let client_id = connection_end.client_id();
		let client_type =
			client_id.as_str().rsplit_once('-').map(|(client_type_str, ..)| client_type_str);
		match client_type {
			Some(ty) if ty.contains("mock") => {
				let mock_client = MockClientWeightInfo::<T>::new();
				mock_client.conn_open_ack_mock()
			},
			_ => Weight::default(),
		}
	}

	fn conn_open_confirm(msg_conn_open_confirm: MsgConnectionOpenConfirm) -> Weight {
		let connection_id = msg_conn_open_confirm.conn_id_on_b;
		let ctx = Context::<T>::new();
		let connection_end = ctx.connection_end(&connection_id).unwrap_or_default();
		let client_id = connection_end.client_id();
		let client_type = <Clients<T>>::get(ClientTypePath(client_id.clone()))
			.expect(&format!("cannt find client type by {}", client_id));
		match client_type.as_str() {
			MOCK_CLIENT_TYPE => {
				let mock_client = MockClientWeightInfo::<T>::new();
				mock_client.conn_open_confirm_mock()
			},
			_ => Weight::default(),
		}
	}

	fn channel_open_init(msg_channel_open_init: MsgChannelOpenInit) -> Weight {
		let cb: Box<dyn CallbackWeight> =
			WeightRouter::<T>::get_weight(&msg_channel_open_init.port_id_on_a)
				.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_chan_open_init();
		let lc_verification_weight = match msg_channel_open_init.connection_hops_on_a.get(0) {
			Some(connection_id) => {
				let ctx = Context::<T>::new();
				let connection_end = ctx.connection_end(connection_id).unwrap_or_default();
				let client_id = connection_end.client_id();
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("mock") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.channel_open_init_mock()
					},
					_ => Weight::default(),
				}
			},
			None => Weight::default(),
		};

		cb_weight.saturating_add(lc_verification_weight)
	}

	fn channel_open_try(msg_channel_open_try: MsgChannelOpenTry) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_channel_open_try.port_id_on_b)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_chan_open_try();
		let lc_verification_weight = match msg_channel_open_try.connection_hops_on_b.get(0) {
			Some(connection_id) => {
				let ctx = Context::<T>::new();
				let connection_end = ctx.connection_end(connection_id).unwrap_or_default();
				let client_id = connection_end.client_id();
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("mock") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.channel_open_try_mock()
					},
					_ => Weight::default(),
				}
			},
			None => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn channel_open_ack(msg_channel_open_ack: MsgChannelOpenAck) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_channel_open_ack.port_id_on_a)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_chan_open_ack(
			&msg_channel_open_ack.port_id_on_a,
			&msg_channel_open_ack.chan_id_on_a,
		);
		let lc_verification_weight = match channel_client::<T>(
			&msg_channel_open_ack.chan_id_on_a,
			&msg_channel_open_ack.port_id_on_a,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("mock") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.channel_open_ack_mock()
					},
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn channel_open_confirm(msg_channel_open_confirm: MsgChannelOpenConfirm) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_channel_open_confirm.port_id_on_b)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_chan_open_confirm(
			&msg_channel_open_confirm.port_id_on_b,
			&msg_channel_open_confirm.chan_id_on_b,
		);
		let lc_verification_weight = match channel_client::<T>(
			&msg_channel_open_confirm.chan_id_on_b,
			&msg_channel_open_confirm.port_id_on_b,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("mock") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.channel_open_confirm_mock()
					},
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn channel_close_init(msg_channel_close_init: MsgChannelCloseInit) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_channel_close_init.port_id_on_a)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_chan_close_init(
			&msg_channel_close_init.port_id_on_a,
			&msg_channel_close_init.chan_id_on_a,
		);
		let lc_verification_weight = match channel_client::<T>(
			&msg_channel_close_init.chan_id_on_a,
			&msg_channel_close_init.port_id_on_a,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("mock") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.channel_close_init_mock()
					},
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn channel_close_confirm(msg_channel_close_confirm: MsgChannelCloseConfirm) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_channel_close_confirm.port_id_on_b)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_chan_close_confirm(
			&msg_channel_close_confirm.port_id_on_b,
			&msg_channel_close_confirm.chan_id_on_b,
		);
		let lc_verification_weight = match channel_client::<T>(
			&msg_channel_close_confirm.chan_id_on_b,
			&msg_channel_close_confirm.port_id_on_b,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("mock") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.channel_close_confirm_mock()
					},
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn recv_packet(msg_recv_packet: MsgRecvPacket) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_recv_packet.packet.port_on_b)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_recv_packet(&msg_recv_packet.packet);
		let lc_verification_weight = match channel_client::<T>(
			&msg_recv_packet.packet.chan_on_b,
			&msg_recv_packet.packet.port_on_b,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("tendermint") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.recv_packet_mock()
					},
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn ack_packet(msg_ack_packet: MsgAcknowledgement) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_ack_packet.packet.port_on_b)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight =
			cb.on_acknowledgement_packet(&msg_ack_packet.packet, &msg_ack_packet.acknowledgement);
		let lc_verification_weight = match channel_client::<T>(
			&msg_ack_packet.packet.chan_on_b,
			&msg_ack_packet.packet.port_on_b,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("tendermint") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.ack_packet_mock()
					},
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn timeout_packet(msg_timeout_packet: MsgTimeout) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_timeout_packet.packet.port_on_b)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_timeout_packet(&msg_timeout_packet.packet);
		let lc_verification_weight = match channel_client::<T>(
			&msg_timeout_packet.packet.chan_on_b,
			&msg_timeout_packet.packet.port_on_b,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("tendermint") => {
						let mock_client = MockClientWeightInfo::<T>::new();
						mock_client.timeout_packet_mock()
					},
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}

	fn timeout_on_close_packet(msg_timout_onclose_packet: MsgTimeoutOnClose) -> Weight {
		let cb = WeightRouter::<T>::get_weight(&msg_timout_onclose_packet.packet.port_on_b)
			.unwrap_or_else(|| Box::new(()));
		let cb_weight = cb.on_timeout_packet(&msg_timout_onclose_packet.packet);
		let lc_verification_weight = match channel_client::<T>(
			&msg_timout_onclose_packet.packet.chan_on_b,
			&msg_timout_onclose_packet.packet.port_on_b,
		) {
			Ok(client_id) => {
				let client_type = client_id
					.as_str()
					.rsplit_once('-')
					.map(|(client_type_str, ..)| client_type_str);
				match client_type {
					Some(ty) if ty.contains("tendermint") => Weight::default(),
					_ => Weight::default(),
				}
			},
			Err(_) => Weight::default(),
		};
		cb_weight.saturating_add(lc_verification_weight)
	}
}

pub struct WeightRouter<T: Config>(PhantomData<T>);

impl<T: Config> WeightRouter<T> {
	pub fn get_weight(port_id: &PortId) -> Option<Box<dyn CallbackWeight>> {
		match port_id.as_str() {
			ibc::applications::transfer::PORT_ID_STR => Some(Box::new(())),
			_ => None,
		}
	}
}

/// Get client id for a port and channel combination
pub fn channel_client<T: Config>(
	channel_id: &ChannelId,
	port_id: &PortId,
) -> Result<ClientId, Error<T>> {
	for (connection_id, channels) in ChannelsConnection::<T>::iter() {
		if channels.contains(&(port_id.clone(), channel_id.clone())) {
			if let Some((client_id, ..)) = ConnectionClient::<T>::iter()
				.find(|(.., connection_ids)| connection_ids == &connection_id)
			{
				return Ok(client_id);
			}
		}
	}
	Err(Error::<T>::Other)
}

#[allow(dead_code)]
pub(crate) fn deliver<T: Config + Send + Sync>(
	msgs: &[ibc_proto::google::protobuf::Any],
) -> Weight {
	msgs.into_iter()
		.filter_map(|msg| {
			let msg: Option<MsgEnvelope> = msg.clone().try_into().ok();
			msg
		})
		.fold(Weight::default(), |acc, msg| {
			// Add benchmarked weight for that message type
			// Add benchmarked weight for module callback
			let temp = match msg {
				MsgEnvelope::Client(msgs) => match msgs {
					ClientMsg::CreateClient(msg) => <T as Config>::WeightInfo::create_client(msg),
					ClientMsg::UpdateClient(msg) => <T as Config>::WeightInfo::update_client(msg),
					ClientMsg::UpgradeClient(msg) => <T as Config>::WeightInfo::upgrade_client(msg),
					ClientMsg::Misbehaviour(msg) => <T as Config>::WeightInfo::misbehaviour(msg),
				},
				MsgEnvelope::Connection(msgs) => match msgs {
					ConnectionMsg::OpenInit(msg) => <T as Config>::WeightInfo::conn_open_init(msg),
					ConnectionMsg::OpenTry(msg) => <T as Config>::WeightInfo::conn_try_open(msg),
					ConnectionMsg::OpenAck(msg) => <T as Config>::WeightInfo::conn_open_ack(msg),
					ConnectionMsg::OpenConfirm(msg) => {
						<T as Config>::WeightInfo::conn_open_confirm(msg)
					},
				},
				MsgEnvelope::Channel(msgs) => match msgs {
					ChannelMsg::OpenInit(msg) => <T as Config>::WeightInfo::channel_open_init(msg),
					ChannelMsg::OpenTry(msg) => <T as Config>::WeightInfo::channel_open_try(msg),
					ChannelMsg::OpenAck(msg) => <T as Config>::WeightInfo::channel_open_ack(msg),
					ChannelMsg::OpenConfirm(msg) => {
						<T as Config>::WeightInfo::channel_open_confirm(msg)
					},
					ChannelMsg::CloseInit(msg) => {
						<T as Config>::WeightInfo::channel_close_init(msg)
					},
					ChannelMsg::CloseConfirm(msg) => {
						<T as Config>::WeightInfo::channel_close_confirm(msg)
					},
				},
				MsgEnvelope::Packet(msg) => match msg {
					PacketMsg::Recv(msg) => <T as Config>::WeightInfo::recv_packet(msg),
					PacketMsg::Ack(msg) => <T as Config>::WeightInfo::ack_packet(msg),
					PacketMsg::Timeout(msg) => <T as Config>::WeightInfo::timeout_packet(msg),
					PacketMsg::TimeoutOnClose(msg) => {
						<T as Config>::WeightInfo::timeout_on_close_packet(msg)
					},
				},
			};
			acc.saturating_add(temp)
		})
}
