pub mod test_util {
	use ibc_proto::ibc::core::channel::v1::MsgChannelCloseInit as RawMsgChannelCloseInit;

	use crate::tests::common::get_dummy_bech32_account;
	use ibc::core::ics24_host::identifier::{ChannelId, PortId};

	/// Returns a dummy `RawMsgChannelCloseInit`, for testing only!
	pub fn get_dummy_raw_msg_chan_close_init() -> RawMsgChannelCloseInit {
		RawMsgChannelCloseInit {
			port_id: PortId::transfer().to_string(),
			channel_id: ChannelId::default().to_string(),
			signer: get_dummy_bech32_account(),
		}
	}
}

#[cfg(test)]
mod tests {

	use ibc::core::ics04_channel::{
		context::ChannelReader,
		msgs::{chan_close_init::MsgChannelCloseInit, ChannelMsg},
	};

	use crate::tests::connection::common::test_util::get_dummy_raw_counterparty;
	use ibc::core::{
		ics03_connection::{
			connection::{
				ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
			},
			version::get_compatible_versions,
		},
		ics04_channel::{
			channel::{ChannelEnd, Counterparty, Order, State as ChannelState},
			Version,
		},
		ics24_host::identifier::{ClientId, ConnectionId},
		ics26_routing::{handler::dispatch, msgs::MsgEnvelope},
	};

	use super::test_util::get_dummy_raw_msg_chan_close_init;
	use crate::{
		mock::{new_test_ext, System, Test as PalletIbcTest},
		Context,
	};
	use ibc::{mock::client_state::client_type as mock_client_type, timestamp::ZERO_DURATION};

	#[test]
	#[ignore]
	fn chan_close_init_event_height() {
		new_test_ext().execute_with(|| {
			let client_id = ClientId::new(mock_client_type(), 24).unwrap();
			let conn_id = ConnectionId::new(2);

			let conn_end = ConnectionEnd::new(
				ConnectionState::Open,
				client_id.clone(),
				ConnectionCounterparty::try_from(get_dummy_raw_counterparty()).unwrap(),
				get_compatible_versions(),
				ZERO_DURATION,
			);

			let msg_chan_close_init =
				MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

			let chan_end = ChannelEnd::new(
				ChannelState::Open,
				Order::default(),
				Counterparty::new(
					msg_chan_close_init.port_id_on_a.clone(),
					Some(msg_chan_close_init.chan_id_on_a.clone()),
				),
				vec![conn_id.clone()],
				Version::default(),
			);

			System::set_block_number(20);
			let mut context = {
				let default_context = Context::<PalletIbcTest>::new();
				let client_consensus_state_height = default_context.host_height().unwrap();

				default_context
					.with_client(&client_id, client_consensus_state_height)
					.with_connection(conn_id, conn_end)
					.with_channel(
						msg_chan_close_init.port_id_on_a.clone(),
						msg_chan_close_init.chan_id_on_a.clone(),
						chan_end,
					)
			};

			dispatch(
				&mut context,
				MsgEnvelope::Channel(ChannelMsg::CloseInit(msg_chan_close_init)),
			)
			.unwrap();
		})
	}
}
