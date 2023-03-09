pub mod test_util {
	use crate::tests::{
		channel::common::test_util::get_dummy_raw_channel_end, common::get_dummy_bech32_account,
	};
	use ibc::core::ics24_host::identifier::PortId;
	use ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;

	#[allow(dead_code)]
	/// Returns a dummy `RawMsgChannelOpenInit`, for testing only!
	pub fn get_dummy_raw_msg_chan_open_init() -> RawMsgChannelOpenInit {
		RawMsgChannelOpenInit {
			port_id: PortId::transfer().to_string(),
			channel: Some(get_dummy_raw_channel_end()),
			signer: get_dummy_bech32_account(),
		}
	}
}
