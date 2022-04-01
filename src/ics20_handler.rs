use super::*;

use crate::alloc::string::ToString;
use event::primitive::{
	ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height,
	Packet as IbcPacket, PortId, Timestamp,
};
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as Ics20Error, msgs::denom_trace,
	},
	core::ics04_channel::{msgs::acknowledgement::MsgAcknowledgement, packet::Packet},
};
use ibc_proto::ibc::apps::transfer::v2::FungibleTokenPacketData as IBCFungibleTokenPacketData;

use frame_support::{
	sp_runtime::traits::{AtLeast32BitUnsigned, CheckedConversion},
	sp_std::fmt::Debug,
	traits::{tokens::fungibles, Currency, ExistenceRequirement::AllowDeath},
	PalletId,
};
use sp_runtime::traits::AccountIdConversion;

// use ibc_proto::ibc::apps::transfer::v2::FungibleTokenPacketData;
use ibc_proto::ibc::core::channel::v1::{acknowledgement::Response, Acknowledgement};

fn generate_escrow_account<T: Config>(
	channel_id: ibc::core::ics24_host::identifier::ChannelId,
) -> T::AccountId {
	let channel_id_number =
		channel_id.as_str().strip_prefix("channel-").unwrap().parse::<u64>().unwrap();
	let channel_id_number = format!("cha{:>05}", channel_id_number).as_bytes().to_vec();

	let mut temp_value = [0u8; 8];
	temp_value.copy_from_slice(&channel_id_number);
	let escrow_account: T::AccountId = PalletId(temp_value).into_account();

	escrow_account
}

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
pub fn handle_transfer<Ctx, T: Config>(ctx: &Ctx, packet: Packet) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	// // NOTE: denomination and hex hash correctness checked during msg.ValidateBasic
	// fullDenomPath := token.Denom

	// var err error

	// // deconstruct the token denomination into the denomination trace info
	// // to determine if the sender is the source chain
	// if strings.HasPrefix(token.Denom, "ibc/") {
	// 	fullDenomPath, err = k.DenomPathFromHash(ctx, token.Denom)
	// 	if err != nil {
	// 		return err
	// 	}
	// }

	//TODO: add telemetry
	// labels := []metrics.Label{
	// 	telemetry.NewLabel(coretypes.LabelDestinationPort, destinationPort),
	// 	telemetry.NewLabel(coretypes.LabelDestinationChannel, destinationChannel),
	// }

	// // NOTE: SendTransfer simply sends the denomination as it exists on its own
	// // chain inside the packet data. The receiving chain will perform denom
	// // prefixing as necessary.

	// if types.SenderChainIsSource(sourcePort, sourceChannel, fullDenomPath) {
	// 	labels = append(labels, telemetry.NewLabel(coretypes.LabelSource, "true"))

	// 	// create the escrow address for the tokens
	// 	escrowAddress := types.GetEscrowAddress(sourcePort, sourceChannel)

	// 	// escrow source tokens. It fails if balance insufficient.
	// 	if err := k.bankKeeper.SendCoins(
	// 		ctx, sender, escrowAddress, sdk.NewCoins(token),
	// 	); err != nil {
	// 		return err
	// 	}

	// } else {
	// 	labels = append(labels, telemetry.NewLabel(coretypes.LabelSource, "false"))

	// 	// transfer the coins to the module account and burn them
	// 	if err := k.bankKeeper.SendCoinsFromAccountToModule(
	// 		ctx, sender, types.ModuleName, sdk.NewCoins(token),
	// 	); err != nil {
	// 		return err
	// 	}

	// 	if err := k.bankKeeper.BurnCoins(
	// 		ctx, types.ModuleName, sdk.NewCoins(token),
	// 	); err != nil {
	// 		// NOTE: should not happen as the module account was
	// 		// retrieved on the step above and it has enough balace
	// 		// to burn.
	// 		panic(fmt.Sprintf("cannot burn coins after a successful send to a module account: %v", err))
	// 	}
	// }

	//TODO: get data from packet
	let source_channel = packet.source_channel.clone();
	let source_port = packet.source_port.clone();
	let denomination = String::new(); // TODO
	let amount: u128 = 10; // TODO
	let sender: T::AccountId = T::AccountId::default(); // TODO

	// let destination_channel = ChannelId::from(_value.packet.destination_channel);
	// let destination_port = PortId::from(_value.packet.destination_port);
	// let timeout_timestamp= Timestamp::from(_value.packet.timeout_timestamp);
	// let timeout_height = Height::from(_value.packet.timeout_height);
	// let data = packet.data;

	//TODO: get FungibleTokenPacketData from packet data
	// let fungible_token_packet_data = FungibleTokenPacketData::decode(&mut
	// data.as_slice()).unwrap();

	// let token_id = fungible_token_packet_data.token_id;
	// let sender= fungible_token_packet_data.sender_chain_id;
	// let receiver = fungible_token_packet_data.receiver_chain_id;

	let prefix = format!("{}/{}", source_port, source_channel);
	let source = denomination.starts_with(&prefix);

	if source {
		// determine escrow account
		let escrow_account = generate_escrow_account::<T>(source_channel.clone());
		<ChannelEscrowAddresses<T>>::insert(
			ChannelId::from(source_channel),
			escrow_account.clone(),
		);

		// escrow source tokens (assumed to fail if balance insufficient)
		// bank.TransferCoins(sender, escrowAccount, denomination, amount)

		// TODO
		// how to deail with denomination
		let amount = amount.checked_into().unwrap(); // TODO: unwrap()
		let ret = T::Currency::transfer(&sender, &escrow_account, amount, AllowDeath).unwrap();
		// TODO: unwrap()
	} else {
		// todo
		// receiver is source chain, burn vouchers
		// bank.BurnCoins(sender, denomination, amount)
		// todo how to deail with denomination <> asset_id
		// todo Assetid is default
		let amount = amount.checked_into().unwrap(); // TODO : unwrap()
		let ret = <T::Assets as fungibles::Mutate<T::AccountId>>::burn_from(
			T::AssetId::default(),
			&sender,
			amount,
		)
		.unwrap(); // TODO: unwrap()
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
	packet: Packet,
	data: FungibleTokenPacketData<T>,
) -> Result<FungibleTokenPacketAcknowledgement, Ics20Error>
where
	Ctx: Ics20Context,
{
	// // validate packet data upon receiving
	// if err := data.ValidateBasic(); err != nil {
	// 	return err
	// }

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
	// 	return sdkerrors.Wrapf(types.ErrInvalidAmount, "unable to parse transfer amount (%s) into sdk.Int", data.Amount)
	// }

	// labels := []metrics.Label{
	// 	telemetry.NewLabel(coretypes.LabelSourcePort, packet.GetSourcePort()),
	// 	telemetry.NewLabel(coretypes.LabelSourceChannel, packet.GetSourceChannel()),
	// }

	// // This is the prefix that would have been prefixed to the denomination
	// // on sender chain IF and only if the token originally came from the
	// // receiving chain.
	// //
	// // NOTE: We use SourcePort and SourceChannel here, because the counterparty
	// // chain would have prefixed with DestPort and DestChannel when originally
	// // receiving this coin as seen in the "sender chain is the source" condition.

	// if types.ReceiverChainIsSource(packet.GetSourcePort(), packet.GetSourceChannel(), data.Denom) {
	// 	// sender chain is not the source, unescrow tokens

	// 	// remove prefix added by sender chain
	// 	voucherPrefix := types.GetDenomPrefix(packet.GetSourcePort(), packet.GetSourceChannel())
	// 	unprefixedDenom := data.Denom[len(voucherPrefix):]

	// 	// coin denomination used in sending from the escrow address
	// 	denom := unprefixedDenom

	// 	// The denomination used to send the coins is either the native denom or the hash of the path
	// 	// if the denomination is not native.
	// 	denomTrace := types.ParseDenomTrace(unprefixedDenom)
	// 	if denomTrace.Path != "" {
	// 		denom = denomTrace.IBCDenom()
	// 	}
	// 	token := sdk.NewCoin(denom, transferAmount)

	// 	// unescrow tokens
	// 	escrowAddress := types.GetEscrowAddress(packet.GetDestPort(), packet.GetDestChannel())
	// 	if err := k.bankKeeper.SendCoins(ctx, escrowAddress, receiver, sdk.NewCoins(token)); err != nil {
	// 		// NOTE: this error is only expected to occur given an unexpected bug or a malicious
	// 		// counterparty module. The bug may occur in bank or any part of the code that allows
	// 		// the escrow address to be drained. A malicious counterparty module could drain the
	// 		// escrow address by allowing more tokens to be sent back then were escrowed.
	// 		return sdkerrors.Wrap(err, "unable to unescrow tokens, this may be caused by a malicious counterparty module or a bug: please open an issue on counterparty module")
	// 	}

	// 	defer func() {
	// 		if transferAmount.IsInt64() {
	// 			telemetry.SetGaugeWithLabels(
	// 				[]string{"ibc", types.ModuleName, "packet", "receive"},
	// 				float32(transferAmount.Int64()),
	// 				[]metrics.Label{telemetry.NewLabel(coretypes.LabelDenom, unprefixedDenom)},
	// 			)
	// 		}

	// 		telemetry.IncrCounterWithLabels(
	// 			[]string{"ibc", types.ModuleName, "receive"},
	// 			1,
	// 			append(
	// 				labels, telemetry.NewLabel(coretypes.LabelSource, "true"),
	// 			),
	// 		)
	// 	}()

	// 	return nil
	// }

	// // sender chain is the source, mint vouchers

	// // since SendPacket did not prefix the denomination, we must prefix denomination here
	// sourcePrefix := types.GetDenomPrefix(packet.GetDestPort(), packet.GetDestChannel())
	// // NOTE: sourcePrefix contains the trailing "/"
	// prefixedDenom := sourcePrefix + data.Denom

	// // construct the denomination trace from the full raw denomination
	// denomTrace := types.ParseDenomTrace(prefixedDenom)

	// traceHash := denomTrace.Hash()
	// if !k.HasDenomTrace(ctx, traceHash) {
	// 	k.SetDenomTrace(ctx, denomTrace)
	// }
	//TODO: rust implementation

	// voucherDenom := denomTrace.IBCDenom()
	// ctx.EventManager().EmitEvent(
	// 	sdk.NewEvent(
	// 		types.EventTypeDenomTrace,
	// 		sdk.NewAttribute(types.AttributeKeyTraceHash, traceHash.String()),
	// 		sdk.NewAttribute(types.AttributeKeyDenom, voucherDenom),
	// 	),
	// )
	// voucher := sdk.NewCoin(voucherDenom, transferAmount)

	// // mint new tokens if the source of the transfer is the same chain
	// if err := k.bankKeeper.MintCoins(
	// 	ctx, types.ModuleName, sdk.NewCoins(voucher),
	// ); err != nil {
	// 	return err
	// }

	// // send to receiver
	// if err := k.bankKeeper.SendCoinsFromModuleToAccount(
	// 	ctx, types.ModuleName, receiver, sdk.NewCoins(voucher),
	// ); err != nil {
	// 	return err
	// }

	// defer func() {
	// 	if transferAmount.IsInt64() {
	// 		telemetry.SetGaugeWithLabels(
	// 			[]string{"ibc", types.ModuleName, "packet", "receive"},
	// 			float32(transferAmount.Int64()),
	// 			[]metrics.Label{telemetry.NewLabel(coretypes.LabelDenom, data.Denom)},
	// 		)
	// 	}

	// 	telemetry.IncrCounterWithLabels(
	// 		[]string{"ibc", types.ModuleName, "receive"},
	// 		1,
	// 		append(
	// 			labels, telemetry.NewLabel(coretypes.LabelSource, "false"),
	// 		),
	// 	)
	// }()

	// return nil

	let data = packet.data.clone();
	let data = FungibleTokenPacketData::<T>::decode(&mut &*data).unwrap();
	// construct default acknowledgement of success
	let mut ack = FungibleTokenPacketAcknowledgement::new();
	let prefix = format!("{}/{}", packet.source_port, packet.source_channel).as_bytes().to_vec();
	// we are the source if the packets were prefixed by the sending chain
	let source = data.denomination.starts_with(&prefix);
	// let amount_unwrapped = data.amount.checked_into().ok_or(Error::<T>::AmountOverflow)?;
	if source {
		// todo
		// receiver is source chain: unescrow tokens
		// determine escrow account
		// escrowAccount = channelEscrowAddresses[packet.destChannel]
		let escrow_account = generate_escrow_account::<T>(packet.source_channel.clone());
		<ChannelEscrowAddresses<T>>::insert(
			ChannelId::from(packet.source_channel),
			escrow_account.clone(),
		);
		// unescrow tokens to receive (assumed to fail if balance insufficient)
		// err = bank.TransferCoins(escrowAccount, data.receiver,
		// data.denomination.slice(len(prefix)), data.amount) if (err !== nil)
		// ack = FungibleTokenPacketAcknowledgement{false, "transfer coins failed"}
		// how to deail with denomination
		let amount = data.amount.checked_into().unwrap();
		let result = T::Currency::transfer(&escrow_account, &data.receiver, amount, AllowDeath);
		match result {
			Ok(_) => {},
			Err(_err) => {
				ack = FungibleTokenPacketAcknowledgement::Err(FungibleTokenPacketError {
					error: "transfer coin failed".to_string(),
				})
			},
		}
	} else {
		// todo
		// prefix = "{packet.destPort}/{packet.destChannel}/"
		let prefix = format!("{}/{}", packet.destination_port, packet.destination_channel);
		// prefixedDenomination = prefix + data.denomination
		let denomination = String::from_utf8(data.denomination).unwrap();
		let _prefixed_denomination = format!("{}{}", prefix, denomination);
		// sender was source, mint vouchers to receiver (assumed to fail if balance insufficient)
		// err = bank.MintCoins(data.receiver, prefixedDenomination, data.amount)
		// todo how to deail with asset_id
		// todo asset id is default
		let amount = data.amount.checked_into().unwrap();
		let result = <T::Assets as fungibles::Mutate<T::AccountId>>::mint_into(
			T::AssetId::default(),
			&data.receiver,
			amount,
		);
		match result {
			Ok(()) => {},
			Err(_) => {
				ack = FungibleTokenPacketAcknowledgement::Err(FungibleTokenPacketError {
					error: "mint coins failed".to_string(),
				})
			},
		}
	}

	Ok(ack)
}

/// OnTimeoutPacket refunds the sender since the original packet sent was
/// never received and has been timed out.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L350
pub fn handle_timeout_packet<Ctx, T: Config>(
	ctx: &Ctx,
	packet: Packet,
	data: FungibleTokenPacketData<T>,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	log::trace!("in ics20_handler : handle timeout packet !");
	refund_packet_token(ctx, packet, data)
	// Ok(())
}

/// onAcknowledgePacket is called by the routing module when a packet sent by this module has been
/// acknowledged. OnAcknowledgementPacket responds to the the success or failure of a packet
/// acknowledgement written on the receiving chain. If the acknowledgement
/// was a success then nothing occurs. If the acknowledgement failed, then
/// the sender is refunded their tokens using the refundPacketToken function.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L337
pub fn handle_ack_packet<Ctx, T: Config>(
	ctx: &Ctx,
	packet: Packet,
	data: FungibleTokenPacketData<T>,
	acknowledgement: Acknowledgement,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	// let acknowledgement = String::from_utf8(acknowledgement).unwrap();
	// let acknowledgement: FungibleTokenPacketAcknowledgement =
	// 	serde_json::from_str(&acknowledgement).unwrap();
	// // if the transfer failed, refund the token
	// match acknowledgement {
	// 	FungibleTokenPacketAcknowledgement::Err(_ack) => {
	// 		return refund_packet_token(ctx, packet, data)
	// 	},
	// 	_ => unimplemented!(),
	// }

	let response = acknowledgement.response.ok_or(Ics20Error::acknowledgement_response_empty())?;

	match response {
		Response::Error(e) => {
			log::trace!("in ics20_handler : handle ack packet error >> {:?}", e);
			return refund_packet_token(ctx, packet, data);
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
	packet: Packet,
	data: FungibleTokenPacketData<T>,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{

	// NOTE: packet data type already checked in handler.go

	// parse the denomination from the full denom path
	// trace := types.ParseDenomTrace(data.Denom)

	// // parse the transfer amount
	// transferAmount, ok := sdk.NewIntFromString(data.Amount)
	// if !ok {
	// 	return sdkerrors.Wrapf(types.ErrInvalidAmount, "unable to parse transfer amount (%s) into sdk.Int", data.Amount)
	// }
	// token := sdk.NewCoin(trace.IBCDenom(), transferAmount)

	// // decode the sender address
	// sender, err := sdk.AccAddressFromBech32(data.Sender)
	// if err != nil {
	// 	return err
	// }

	// if types.SenderChainIsSource(packet.GetSourcePort(), packet.GetSourceChannel(), data.Denom) {
	// 	// unescrow tokens back to sender
	// 	escrowAddress := types.GetEscrowAddress(packet.GetSourcePort(), packet.GetSourceChannel())
	// 	if err := k.bankKeeper.SendCoins(ctx, escrowAddress, sender, sdk.NewCoins(token)); err != nil {
	// 		// NOTE: this error is only expected to occur given an unexpected bug or a malicious
	// 		// counterparty module. The bug may occur in bank or any part of the code that allows
	// 		// the escrow address to be drained. A malicious counterparty module could drain the
	// 		// escrow address by allowing more tokens to be sent back then were escrowed.
	// 		return sdkerrors.Wrap(err, "unable to unescrow tokens, this may be caused by a malicious counterparty module or a bug: please open an issue on counterparty module")
	// 	}

	// 	return nil
	// }

	// // mint vouchers back to sender
	// if err := k.bankKeeper.MintCoins(
	// 	ctx, types.ModuleName, sdk.NewCoins(token),
	// ); err != nil {
	// 	return err
	// }

	// if err := k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ModuleName, sender, sdk.NewCoins(token)); err != nil {
	// 	panic(fmt.Sprintf("unable to send coins from module to account despite previously minting coins to module account: %v", err))
	// }

	// return nil

	let data = packet.data.clone();
	let data = FungibleTokenPacketData::<T>::decode(&mut &*data).unwrap();
	let prefix = format!("{}/{}", packet.source_port, packet.source_channel).as_bytes().to_vec();
	// we are the source if the denomination is not prefixed
	let source = data.denomination.starts_with(&prefix);
	// let amount_unwrapped = data.amount.checked_into().ok_or(Error::<T>::AmountOverflow)?;
	if source {
		// todo
		// sender was source chain, unescrow tokens back to sender
		// escrowAccount = channelEscrowAddresses[packet.srcChannel]
		let escrow_account = generate_escrow_account::<T>(packet.source_channel.clone());
		<ChannelEscrowAddresses<T>>::insert(
			ChannelId::from(packet.source_channel),
			escrow_account.clone(),
		);
		// bank.TransferCoins(escrowAccount, data.sender, data.denomination, data.amount)
		// todo how to deail with denomination
		let amount = data.amount.checked_into().unwrap();
		T::Currency::transfer(&escrow_account, &data.sender, amount, AllowDeath).unwrap();
	} else {
		// todo
		// receiver was source chain, mint vouchers back to sender
		// bank.MintCoins(data.sender, denomination, data.amount)
		// todo how to deail with denomination
		// todo how to deail with asset id
		let amount = data.amount.checked_into().unwrap();
		<T::Assets as fungibles::Mutate<T::AccountId>>::mint_into(
			T::AssetId::default(),
			&data.receiver,
			amount,
		)
		.unwrap();
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
	// hexHash := denom[len(types.DenomPrefix+"/"):]
	let denom_split = denom.split("/").collect::<Vec<&str>>();
	let hex_hash = denom_split[1];

	// hash, err := types.ParseHexHash(hexHash)
	// if err != nil {
	// 	return "", sdkerrors.Wrap(types.ErrInvalidDenomForTransfer, err.Error())
	// }
	let hash = denom_trace::parse_hex_hash(hex_hash).unwrap();

	// denomTrace, found := k.GetDenomTrace(ctx, hash)
	// if !found {
	// 	return "", sdkerrors.Wrap(types.ErrTraceNotFound, hexHash)
	// }
	let trace = ctx.get_denom_trace(&hash).unwrap();

	// fullDenomPath := denomTrace.GetFullDenomPath()
	// return fullDenomPath, nil
	trace.get_full_denom_path()
	// Ok(full_denom_path)
}
