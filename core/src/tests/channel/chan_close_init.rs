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
