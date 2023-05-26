pub mod test {
	use crate::{callback::IbcTransferModule, mock::Test as PalletIbcTest};
	use ibc::{
		applications::transfer::{
			error::TokenTransferError, msgs::transfer::MsgTransfer,
			relay::send_transfer::send_transfer, PrefixedCoin,
		},
		core::ics04_channel::error::ChannelError,
		handler::HandlerOutputBuilder,
	};

	#[allow(dead_code)]
	pub fn deliver(
		ctx: &mut IbcTransferModule<PalletIbcTest>,
		output: &mut HandlerOutputBuilder<()>,
		msg: MsgTransfer<PrefixedCoin>,
	) -> Result<(), ChannelError> {
		send_transfer(ctx, output, msg)
			.map_err(|e: TokenTransferError| ChannelError::AppModule { description: e.to_string() })
	}
}
pub mod test_util {
	use core::{ops::Add, time::Duration};

	use ibc::{
		applications::transfer::{
			msgs::transfer::MsgTransfer, packet::PacketData, BaseCoin, Coin, PrefixedCoin,
		},
		core::{
			ics04_channel::{
				packet::{Packet, Sequence},
				timeout::TimeoutHeight,
			},
			ics24_host::identifier::{ChannelId, PortId},
		},
		signer::Signer,
		timestamp::Timestamp,
	};
	use primitive_types::U256;

	#[allow(dead_code)]
	pub fn get_dummy_substrate_account() -> String {
		"0x3E5DA34F651595C1257265E30370146E0C94B9FBFA78BDB92893DE367AC792A0".to_string()
	}

	#[allow(dead_code)]
	// Returns a dummy ICS20 `MsgTransfer`. If no `timeout_timestamp` is
	// specified, a timestamp of 10 seconds in the future is used.
	pub fn get_dummy_msg_transfer(
		timeout_height: TimeoutHeight,
		timeout_timestamp: Option<Timestamp>,
	) -> MsgTransfer<PrefixedCoin> {
		let address: Signer = get_dummy_substrate_account().as_str().parse().unwrap();
		MsgTransfer {
			port_on_a: PortId::default(),
			chan_on_a: ChannelId::default(),
			token: BaseCoin { denom: "uatom".parse().unwrap(), amount: U256::from(10).into() }
				.into(),
			sender: address.clone(),
			receiver: address,
			timeout_timestamp_on_b: timeout_timestamp
				.unwrap_or_else(|| Timestamp::now().add(Duration::from_secs(10)).unwrap()),
			timeout_height_on_b: timeout_height,
		}
	}

	#[allow(dead_code)]
	pub fn get_dummy_transfer_packet(msg: MsgTransfer<PrefixedCoin>, sequence: Sequence) -> Packet {
		let coin = Coin { denom: msg.token.denom.clone(), amount: msg.token.amount };

		let data = {
			let data = PacketData {
				token: coin,
				sender: msg.sender.clone(),
				receiver: msg.receiver.clone(),
			};
			serde_json::to_vec(&data).expect("PacketData's infallible Serialize impl failed")
		};

		Packet {
			sequence,
			port_on_a: msg.port_on_a,
			chan_on_a: msg.chan_on_a,
			port_on_b: PortId::default(),
			chan_on_b: ChannelId::default(),
			data,
			timeout_height_on_b: msg.timeout_height_on_b,
			timeout_timestamp_on_b: msg.timeout_timestamp_on_b,
		}
	}
}
