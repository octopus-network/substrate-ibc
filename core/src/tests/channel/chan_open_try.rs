pub mod test_util {

	use ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry as RawMsgChannelOpenTry;

	use crate::tests::{
		channel::common::test_util::get_dummy_raw_channel_end,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};
	use ibc::core::ics24_host::identifier::{ChannelId, PortId};

	use ibc_proto::ibc::core::client::v1::Height;

	/// Returns a dummy `RawMsgChannelOpenTry`, for testing only!
	pub fn get_dummy_raw_msg_chan_open_try(proof_height: u64) -> RawMsgChannelOpenTry {
		#[allow(deprecated)]
		RawMsgChannelOpenTry {
			port_id: PortId::transfer().to_string(),
			previous_channel_id: ChannelId::default().to_string(),
			channel: Some(get_dummy_raw_channel_end()),
			counterparty_version: "ics20-1".to_string(),
			proof_init: get_dummy_proof(),
			proof_height: Some(Height { revision_number: 0, revision_height: proof_height }),
			signer: get_dummy_bech32_account(),
		}
	}
}
