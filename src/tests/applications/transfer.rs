pub mod test {
	use ibc::{
		applications::transfer::{
			context::{cosmos_adr028_escrow_address, on_chan_open_try},
			error::Error as Ics20Error,
			msgs::transfer::MsgTransfer,
			relay::send_transfer::send_transfer,
			PrefixedCoin,
		},
		core::{
			ics04_channel::{
				channel::{Counterparty, Order},
				error::Error,
				Version,
			},
			ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		},
		handler::HandlerOutputBuilder,
		test_utils::{get_dummy_transfer_module, DummyTransferModule},
	};
	use pallet_ics20_transfer::ics20_callback::IbcTransferModule;
	use crate::mock::Test as PalletIbcTest;

	pub fn deliver(
		ctx: &mut IbcTransferModule<PalletIbcTest>,
		output: &mut HandlerOutputBuilder<()>,
		msg: MsgTransfer<PrefixedCoin>,
	) -> Result<(), Error> {
		send_transfer(ctx, output, msg).map_err(|e: Ics20Error| Error::app_module(e.to_string()))
	}
}
pub mod test_util {
	use core::{ops::Add, time::Duration};

	use ibc::{
		applications::transfer::{
			msgs::transfer::MsgTransfer, packet::PacketData, BaseCoin, Coin, PrefixedCoin,
		},
		bigint::U256,
		core::{
			ics04_channel::{
				packet::{Packet, Sequence},
				timeout::TimeoutHeight,
			},
			ics24_host::identifier::{ChannelId, PortId},
		},
		signer::Signer,
		test_utils::get_dummy_bech32_account,
		timestamp::Timestamp,
	};

	pub fn get_dummy_substrate_account() -> String {
		"0x3E5DA34F651595C1257265E30370146E0C94B9FBFA78BDB92893DE367AC792A0".to_string()
	}

	// Returns a dummy ICS20 `MsgTransfer`. If no `timeout_timestamp` is
	// specified, a timestamp of 10 seconds in the future is used.
	pub fn get_dummy_msg_transfer(
		timeout_height: TimeoutHeight,
		timeout_timestamp: Option<Timestamp>,
	) -> MsgTransfer<PrefixedCoin> {
		let address: Signer = get_dummy_substrate_account().as_str().parse().unwrap();
		MsgTransfer {
			source_port: PortId::transfer(),
			source_channel: ChannelId::default(),
			token: BaseCoin { denom: "DEMO".parse().unwrap(), amount: U256::from(10).into() }
				.into(),
			sender: address.clone(),
			receiver: address,
			timeout_timestamp: timeout_timestamp
				.unwrap_or_else(|| Timestamp::now().add(Duration::from_secs(10)).unwrap()),
			timeout_height,
		}
	}

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
			source_port: msg.source_port,
			source_channel: msg.source_channel,
			destination_port: PortId::transfer(),
			destination_channel: ChannelId::default(),
			data,
			timeout_height: msg.timeout_height,
			timeout_timestamp: msg.timeout_timestamp,
		}
	}
}
