use super::*;

use crate::alloc::string::ToString;
use event::primitive::{
	ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height, Packet,
	PortId, Timestamp,
};
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context,
		error::Error as Ics20Error,
		msgs::denom_trace::{self, parse_denom_trace},
	},
	core::{
		ics04_channel::{msgs::acknowledgement::MsgAcknowledgement, packet::Packet as IbcPacket},
		ics24_host::identifier::{ChannelId as IbcChannelId, PortId as IbcPortId},
	},
	tx_msg::Msg,
};

use frame_support::{
	sp_runtime::traits::{AtLeast32BitUnsigned, CheckedConversion},
	sp_std::fmt::Debug,
	traits::{tokens::fungibles, Currency, ExistenceRequirement::AllowDeath},
	PalletId,
};
use ibc::applications::ics20_fungible_token_transfer::msgs::fungible_token_packet_data::FungibleTokenPacketData as IBCFungibleTokenPacketData;
use sp_runtime::traits::AccountIdConversion;

use ibc_proto::ibc::core::channel::v1::{acknowledgement::Response, Acknowledgement};

fn generate_escrow_account<T: Config>(
	channel_id: ibc::core::ics24_host::identifier::ChannelId,
) -> Result<T::AccountId, Ics20Error> {
	let channel_id_number = channel_id
		.as_str()
		.strip_prefix("channel-")
		.ok_or(Ics20Error::invalid_split())?
		.parse::<u64>()
		.map_err(|e| Ics20Error::invalid_parse())?;
	let channel_id_number = format!("cha{:>05}", channel_id_number).as_bytes().to_vec();

	let mut temp_value = [0u8; 8];
	temp_value.copy_from_slice(&channel_id_number);
	let escrow_account: T::AccountId = PalletId(temp_value).into_account();

	Ok(escrow_account)
}

fn sender_chain_is_source(
	source_port: &IbcPortId,
	source_channel: &IbcChannelId,
	denom: &str,
) -> bool {
	!receiver_chain_is_source(source_port, source_channel, denom)
}

fn receiver_chain_is_source(
	source_port: &IbcPortId,
	source_channel: &IbcChannelId,
	denom: &str,
) -> bool {
	let voucher_prefix = get_denom_prefix(source_port, source_channel);

	voucher_prefix.starts_with(denom)
}

fn get_denom_prefix(port_id: &IbcPortId, channel_id: &IbcChannelId) -> String {
	format!("{}/{}", port_id, channel_id)
}

// // GetPrefixedDenom returns the denomination with the portID and channelID prefixed
// func GetPrefixedDenom(portID, channelID, baseDenom string) string {
// 	return fmt.Sprintf("%s/%s/%s", portID, channelID, baseDenom)
// }

// // GetTransferCoin creates a transfer coin with the port ID and channel ID
// // prefixed to the base denom.
// func GetTransferCoin(portID, channelID, baseDenom string, amount sdk.Int) sdk.Coin {
// 	denomTrace := ParseDenomTrace(GetPrefixedDenom(portID, channelID, baseDenom))
// 	return sdk.NewCoin(denomTrace.IBCDenom(), amount)
// }
/// handles transfer sending logic. There are 2 possible cases:
///
/// 1. Sender chain is acting as the source zone. The coins are transferred
/// to an escrow address (i.e locked) on the sender chain and then transferred
/// to the receiving chain through IBC TAO logic. It is expected that the
/// receiving chain will mint vouchers to the receiving address.
///
/// 2. Sender chain is acting as the sink zone. The coins (vouchers) are burned
/// on the sender chain and then transferred to the receiving chain though IBC
/// TAO logic. It is expected that the receiving chain, which had previously
/// sent the original denomination, will unescrow the fungible token and send
/// it to the receiving address.
///
/// Another way of thinking of source and sink zones is through the token's
/// timeline. Each send to any chain other than the one it was previously
/// received from is a movement forwards in the token's timeline. This causes
/// trace to be added to the token's history and the destination port and
/// destination channel to be prefixed to the denomination. In these instances
/// the sender chain is acting as the source zone. When the token is sent back
/// to the chain it previously received from, the prefix is removed. This is
/// a backwards movement in the token's timeline and the sender chain
/// is acting as the sink zone.
///
/// Example:
/// These steps of transfer occur: A -> B -> C -> A -> C -> B -> A
///
/// 1. A -> B : sender chain is source zone. Denom upon receiving: 'B/denom'
/// 2. B -> C : sender chain is source zone. Denom upon receiving: 'C/B/denom'
/// 3. C -> A : sender chain is source zone. Denom upon receiving: 'A/C/B/denom'
/// 4. A -> C : sender chain is sink zone. Denom upon receiving: 'C/B/denom'
/// 5. C -> B : sender chain is sink zone. Denom upon receiving: 'B/denom'
/// 6. B -> A : sender chain is sink zone. Denom upon receiving: 'denom'

/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L51
/// pallet-asset lock refer to https://github.com/octopus-network/octopus-pallets/blob/main/appchain/src/lib.rs#L676
/// pallet-asset burn refer to https://github.com/octopus-network/octopus-pallets/blob/main/appchain/src/lib.rs#L731
pub fn handle_transfer<Ctx, T: Config>(ctx: &Ctx, packet: IbcPacket) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	log::info!("🤮ics20_handle transfer packet = {:?}", packet);

	let packet_data: IBCFungibleTokenPacketData =
		serde_json::from_slice(packet.data.clone().as_slice())
			.map_err(Ics20Error::invalid_serde_ibc_fungible_token_packet_data)?;
	log::info!("🤮ics20_handle transfer packet data = {:?}", packet_data);

	// the token denomination to be transferred
	// get the token denomination
	let token_denom = packet_data.clone().denom;
	log::info!("🤮ics20_handle transfer token_denom = {:?}", token_denom);

	// get denom hash example "ibc/hash_number"
	// let denom_trace_hash =
		// token_denom.split_once("/").ok_or(Ics20Error::invalid_split())?.1.as_bytes();
	// log::info!("🤮ics20_handle transfer denom_trace_hash = {:?}", denom_trace_hash);

	// let full_denom_path;
	// if token_denom.starts_with("ibc/") {
		// full_denom_path = ctx.get_denom_trace(denom_trace_hash)?.get_full_denom_path()?;
	// }
	// log::info!("🤮ics20_handle transfer full denom path  = {:?}", full_denom_path);

	// get source_channel_id from packet
	let source_channel = packet.source_channel.clone();
	// get source_port_id  from packet
	let source_port = packet.source_port.clone();

	// convert IBC FungibleTokenPacketData to substrate FungibleTokenPacketData
	let pallet_data: FungibleTokenPacketData<T> = packet_data.into();
	let denomination = token_denom;
	let amount = pallet_data.amount;
	let sender = pallet_data.sender;

	if sender_chain_is_source(&source_port, &source_channel, &denomination) {
		// determine escrow account
		let escrow_account = generate_escrow_account::<T>(source_channel.clone())?;
		<EscrowAddresses<T>>::insert(
			PortId::from(source_port),
			ChannelId::from(source_channel),
			escrow_account.clone(),
		);

		// escrow source tokens (assumed to fail if balance insufficient)
		// bank.TransferCoins(sender, escrowAccount, denomination, amount)

		// TODO
		// how to deail with denomination

		let amount = amount.checked_into().ok_or(Ics20Error::invalid_convert_number())?;
		log::info!("🤮ics20_handle transfer amount = {:?}", amount);

		let ret = T::Currency::transfer(&sender, &escrow_account, amount, AllowDeath)
			.map_err(|_| Ics20Error::invalid_transfer())?;
	} else {
		// todo
		// receiver is source chain, burn vouchers
		// bank.BurnCoins(sender, denomination, amount)
		// todo how to deail with denomination <> asset_id
		// todo Assetid is default

		let amount = amount.checked_into().ok_or(Ics20Error::invalid_convert_number())?;

		log::info!("🤮ics20_handle transfer amount = {:?}", amount);

		let ret = <T::Assets as fungibles::Mutate<T::AccountId>>::burn_from(
			T::AssetId::default(),
			&sender,
			amount,
		)
		.map_err(|_| Ics20Error::invalid_burn())?;
	}

	Ok(())
}

/// OnRecvPacket processes a cross chain fungible token transfer. If the
/// sender chain is the source of minted tokens then vouchers will be minted
/// and sent to the receiving address. Otherwise if the sender chain is sending
/// back tokens this chain originally transferred to it, the tokens are
/// unescrowed and sent to the receiving address.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L189
/// pallet-asset mint refer to https://github.com/octopus-network/octopus-pallets/blob/main/appchain/src/lib.rs#L1068
/// pallet-asset unlock refer to https://github.com/octopus-network/octopus-pallets/blob/main/appchain/src/lib.rs#L1051
pub fn handle_recv_packet<Ctx, T: Config>(
	ctx: &Ctx,
	packet: IbcPacket,
	data: IBCFungibleTokenPacketData,
) -> Result<FungibleTokenPacketAcknowledgement, Ics20Error>
where
	Ctx: Ics20Context,
{
	match data.validate_basic() {
		Ok(()) => (),
		Err(error) => return Err(Ics20Error::invalid_validation(error)),
	}

	// TODO
	// if !k.GetReceiveEnabled(ctx) {
	// 	return types.ErrReceiveDisabled
	// }

	// // decode the receiver address
	// receiver, err := sdk.AccAddressFromBech32(data.Receiver)
	// if err != nil {
	// 	return err
	// }

	// // parse the transfer amount
	// transferAmount, ok := sdk.NewIntFromString(data.Amount)
	// if !ok {
	// 	return sdkerrors.Wrapf(types.ErrInvalidAmount, "unable to parse transfer amount (%s) into
	// sdk.Int", data.Amount) }

	// construct default acknowledgement of success
	let mut ack = FungibleTokenPacketAcknowledgement::new();
	log::info!("🤮ics20_handle handle_recv_packet ack = {:?}", ack);

	if receiver_chain_is_source(
		&packet.source_port.clone(),
		&packet.source_channel.clone(),
		&data.denom,
	) {
		let voucher_prefix = get_denom_prefix(&packet.source_port, &packet.source_channel);
		log::info!("🤮ics20_handle handle_recv_packet voucher_prefix = {:?}", voucher_prefix);
		let unprefixed_denom = &data.denom[0..voucher_prefix.len()];
		log::info!("🤮ics20_handle handle_recv_packet unprefixed_denom = {:?}", unprefixed_denom);

		let mut denom = unprefixed_denom.to_string();
		log::info!("🤮ics20_handle handle_recv_packet denom = {:?}", denom);

		let denom_trace = parse_denom_trace(&denom)?;
		log::info!("🤮ics20_handle handle_recv_packet denom_trace = {:?}", denom_trace);

		if denom_trace.path != "" {
			denom = denom_trace.ibc_denom()?;
		}
		log::info!("🤮ics20_handle handle_recv_packet denom = {:?}", denom);

		// TODO
		// token := sdk.NewCoin(denom, transferAmount)
		// unescrow tokens
		// escrowAddress := types.GetEscrowAddress(packet.GetDestPort(), packet.GetDestChannel())
		// if err := k.bankKeeper.SendCoins(ctx, escrowAddress, receiver, sdk.NewCoins(token)); err
		// != nil { 	// NOTE: this error is only expected to occur given an unexpected bug or a
		// malicious 	// counterparty module. The bug may occur in bank or any part of the code that
		// allows 	// the escrow address to be drained. A malicious counterparty module could drain
		// the 	// escrow address by allowing more tokens to be sent back then were escrowed.
		// 	return sdkerrors.Wrap(err, "unable to unescrow tokens, this may be caused by a malicious
		// counterparty module or a bug: please open an issue on counterparty module") }

		let pallet_data: FungibleTokenPacketData<T> = data.into();
		// log::info!("🤮ics20_handle handle_recv_packet pallet_data = {:?}", pallet_data);
		// create escrow account by source_prot, and source channel
		let escrow_account = generate_escrow_account::<T>(packet.source_channel.clone())?;
		log::info!("🤮ics20_handle handle_recv_packet escrow_account = {:?}", escrow_account);
		<EscrowAddresses<T>>::insert(
			PortId::from(packet.source_port),
			ChannelId::from(packet.source_channel),
			escrow_account.clone(),
		);

		let amount =
			pallet_data.amount.checked_into().ok_or(Ics20Error::invalid_convert_number())?;
		
		log::info!("🤮ics20_handle handle_recv_packet amount = {:?}", amount);

		let result =
			T::Currency::transfer(&escrow_account, &pallet_data.receiver, amount, AllowDeath);
		match result {
			Ok(_) => {},
			Err(_err) =>
				ack = FungibleTokenPacketAcknowledgement::Err(FungibleTokenPacketError {
					error: "transfer coin failed".to_string(),
				}),
		}
	} else {
		let source_prefix = get_denom_prefix(&packet.source_port, &packet.source_channel);
		log::info!("🤮ics20_handle handle_recv_packet source_prefix = {:?}", source_prefix);
		let denomination = data.clone().denom;
		let prefixed_denomination = format!("{}{}", source_prefix, denomination);

		log::info!("🤮ics20_handle handle_recv_packet prefixed_denomination = {:?}", prefixed_denomination);
		let denom_trace = parse_denom_trace(&prefixed_denomination)?;
		log::info!("🤮ics20_handle handle_recv_packet denom_trace = {:?}", denom_trace);

		let trace_hash = denom_trace.hash()?;
		if !ctx.has_denom_trace(&trace_hash) {
			let _ = ctx.set_denom_trace(&denom_trace);
		}

		let voucher_denom = denom_trace.ibc_denom()?;
		log::info!("🤮ics20_handle handle_recv_packet voucher_denom = {:?}", voucher_denom);

		// TODO
		// voucher := sdk.NewCoin(voucherDenom, transferAmount)
		// mint new tokens if the source of the transfer is the same chain
		// if err := k.bankKeeper.MintCoins(
		// 	ctx, types.ModuleName, sdk.NewCoins(voucher),
		// ); err != nil {
		// 	return err
		// }
		// send to receiver
		// if err := k.bankKeeper.SendCoinsFromModuleToAccount(
		// 	ctx, types.ModuleName, receiver, sdk.NewCoins(voucher),
		// ); err != nil {
		// 	return err
		// }

		let pallet_data: FungibleTokenPacketData<T> = data.into();

		let amount =
			pallet_data.amount.checked_into().ok_or(Ics20Error::invalid_convert_number())?;
		
		log::info!("🤮ics20_handle handle_recv_packet amount = {:?}", amount);
		let result = <T::Assets as fungibles::Mutate<T::AccountId>>::mint_into(
			T::AssetId::default(), // TODO
			&pallet_data.receiver,
			amount,
		);
		match result {
			Ok(()) => {},
			Err(_) =>
				ack = FungibleTokenPacketAcknowledgement::Err(FungibleTokenPacketError {
					error: "mint coins failed".to_string(),
				}),
		}
	}

	Ok(ack)
}

/// OnTimeoutPacket refunds the sender since the original packet sent was
/// never received and has been timed out.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L350
pub fn handle_timeout_packet<Ctx, T: Config>(
	ctx: &Ctx,
	packet: IbcPacket,
	data: IBCFungibleTokenPacketData,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	log::trace!("in ics20_handler : handle timeout packet !");
	refund_packet_token::<Ctx, T>(ctx, packet, data)
}

/// onAcknowledgePacket is called by the routing module when a packet sent by this module has been
/// acknowledged. OnAcknowledgementPacket responds to the the success or failure of a packet
/// acknowledgement written on the receiving chain. If the acknowledgement
/// was a success then nothing occurs. If the acknowledgement failed, then
/// the sender is refunded their tokens using the refundPacketToken function.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L337
pub fn handle_ack_packet<Ctx, T: Config>(
	ctx: &Ctx,
	packet: IbcPacket,
	data: IBCFungibleTokenPacketData,
	acknowledgement: Acknowledgement,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	let response = acknowledgement.response.ok_or(Ics20Error::acknowledgement_response_empty())?;

	match response {
		Response::Error(e) => {
			log::trace!("in ics20_handler : handle ack packet error >> {:?}", e);
			return refund_packet_token::<Ctx, T>(ctx, packet, data)
		},
		Response::Result(ret) => Ok(()),
	}
}

/// refundTokens is called by both onAcknowledgePacket, on failure, and onTimeoutPacket, to refund
/// escrowed tokens to the original sender. refundPacketToken will unescrow and send back the tokens
/// back to sender if the sending chain was the source chain. Otherwise, the sent tokens
/// were burnt in the original send so new tokens are minted and sent to
/// the sending address.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L358
fn refund_packet_token<Ctx, T: Config>(
	ctx: &Ctx,
	packet: IbcPacket,
	data: IBCFungibleTokenPacketData,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	let trace = parse_denom_trace(&data.denom)?;
	log::info!("🤮ics20_handle refund_packet_token trace = {:?}", trace);
	// // parse the transfer amount
	// transferAmount, ok := sdk.NewIntFromString(data.Amount)
	// if !ok {
	// 	return sdkerrors.Wrapf(types.ErrInvalidAmount, "unable to parse transfer amount (%s) into
	// sdk.Int", data.Amount) }

	// TODO
	// token := sdk.NewCoin(trace.IBCDenom(), transferAmount)

	// // decode the sender address
	// sender, err := sdk.AccAddressFromBech32(data.Sender)
	// if err != nil {
	// 	return err
	// }

	let pallet_data: FungibleTokenPacketData<T> = data.clone().into();
	// log::info!("🤮ics20_handle refund_packet_token pallet_data = {:?}", pallet_data);

	if sender_chain_is_source(&packet.source_port, &packet.source_channel, &data.denom) {
		let escrow_account = generate_escrow_account::<T>(packet.source_channel.clone())?;
		log::info!("🤮ics20_handle refund_packet_token escrow_account = {:?}", escrow_account);
		<EscrowAddresses<T>>::insert(
			PortId::from(packet.source_port),
			ChannelId::from(packet.source_channel),
			escrow_account.clone(),
		);

		let amount =
			pallet_data.amount.checked_into().ok_or(Ics20Error::invalid_convert_number())?;

		log::info!("🤮ics20_handle refund_packet_token amount = {:?}", amount);
		T::Currency::transfer(&escrow_account, &pallet_data.sender, amount, AllowDeath)
			.map_err(|_| Ics20Error::invalid_transfer())?;
	} else {
		let amount =
			pallet_data.amount.checked_into().ok_or(Ics20Error::invalid_convert_number())?;
		
			log::info!("🤮ics20_handle refund_packet_token amount = {:?}", amount);
		<T::Assets as fungibles::Mutate<T::AccountId>>::mint_into(
			T::AssetId::default(), // TODO
			&pallet_data.receiver,
			amount,
		)
		.map_err(|_| Ics20Error::invalid_mint())?;
	}

	Ok(())
}

/// DenomPathFromHash returns the full denomination path prefix from an ibc denom with a hash
/// component.
/// refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L407
pub fn denom_path_from_hash<Ctx>(ctx: &Ctx, denom: String) -> Result<String, Ics20Error>
where
	Ctx: Ics20Context,
{
	// trim the denomination prefix, by default "ibc/"
	let denom_split = denom.split("/").collect::<Vec<&str>>();

	log::info!("🤮ics20_handle denom_path_from_hash denom_split = {:?}", denom_split);
	let hex_hash = denom_split[1];
	log::info!("🤮ics20_handle denom_path_from_hash hex_hash = {:?}", hex_hash);

	let hash = denom_trace::parse_hex_hash(hex_hash)?;
	log::info!("🤮ics20_handle denom_path_from_hash hash = {:?}", hash);

	let trace = ctx.get_denom_trace(&hash)?;
	log::info!("🤮ics20_handle denom_path_from_hash trace = {:?}", trace);

	trace.get_full_denom_path()
}