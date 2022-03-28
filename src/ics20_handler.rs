use super::*;

use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as Ics20Error,
	},
	core::ics04_channel::packet::Packet,
};
use ibc_proto::ibc::apps::transfer::v2::FungibleTokenPacketData;

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
pub fn handle_transfer<Ctx>(ctx: &Ctx, packet: Packet) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	//TODO: get data from packet
	// let source_channel = ChannelId::from(_value.packet.source_channel);
	// let source_port =  PortId::from(_value.packet.source_port);
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

	//TODO: token state transfaction
	//     prefix = "{sourcePort}/{sourceChannel}/"
	//     // we are the source if the denomination is not prefixed
	//     source = denomination.slice(0, len(prefix)) !== prefix
	//     if source {
	//       // determine escrow account
	//       escrowAccount = channelEscrowAddresses[sourceChannel]
	//       // escrow source tokens (assumed to fail if balance insufficient)
	//       bank.TransferCoins(sender, escrowAccount, denomination, amount)
	//     } else {
	//       // receiver is source chain, burn vouchers
	//       bank.BurnCoins(sender, denomination, amount)
	//     }

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
pub fn handle_recv_packet<Ctx>(
	ctx: &Ctx,
	packet: Packet,
	data: FungibleTokenPacketData,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	//TODO: token state transfaction
	//     prefix = "{packet.sourcePort}/{packet.sourceChannel}/"
	//   // we are the source if the packets were prefixed by the sending chain
	//   source = data.denomination.slice(0, len(prefix)) === prefix
	//   if source {
	//     // receiver is source chain: unescrow tokens
	//     // determine escrow account
	//     escrowAccount = channelEscrowAddresses[packet.destChannel]
	//     // unescrow tokens to receiver (assumed to fail if balance insufficient)
	//     err = bank.TransferCoins(escrowAccount, data.receiver,
	// data.denomination.slice(len(prefix)), data.amount)     if (err !== nil)
	//       ack = FungibleTokenPacketAcknowledgement{false, "transfer coins failed"}
	//   } else {
	//     prefix = "{packet.destPort}/{packet.destChannel}/"
	//     prefixedDenomination = prefix + data.denomination
	//     // sender was source, mint vouchers to receiver (assumed to fail if balance insufficient)
	//     err = bank.MintCoins(data.receiver, prefixedDenomination, data.amount)
	//     if (err !== nil)
	//       ack = FungibleTokenPacketAcknowledgement{false, "mint coins failed"}
	//   }
	Ok(())
}

/// onAcknowledgePacket is called by the routing module when a packet sent by this module has been
/// acknowledged. OnAcknowledgementPacket responds to the the success or failure of a packet
/// acknowledgement written on the receiving chain. If the acknowledgement
/// was a success then nothing occurs. If the acknowledgement failed, then
/// the sender is refunded their tokens using the refundPacketToken function.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L337
pub fn handle_ack_packet<Ctx>(
	ctx: &Ctx,
	packet: Packet,
	data: FungibleTokenPacketData,
	ack: Vec<u8>,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	// switch ack.Response.(type) {
	//     case *channeltypes.Acknowledgement_Error:
	//         return k.refundPacketToken(ctx, packet, data)
	//     default:
	//         // the acknowledgement succeeded on the receiving chain so nothing
	//         // needs to be executed and no error needs to be returned
	//         return nil
	//     }
	Ok(())
}
/// OnTimeoutPacket refunds the sender since the original packet sent was
/// never received and has been timed out.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L350
pub fn handle_timeout_packet<Ctx>(
	ctx: &Ctx,
	packet: Packet,
	data: FungibleTokenPacketData,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	//return k.refundPacketToken(ctx, packet, data)
	Ok(())
}

/// refundTokens is called by both onAcknowledgePacket, on failure, and onTimeoutPacket, to refund
/// escrowed tokens to the original sender. refundPacketToken will unescrow and send back the tokens
/// back to sender if the sending chain was the source chain. Otherwise, the sent tokens
/// were burnt in the original send so new tokens are minted and sent to
/// the sending address.
/// ibc-go implementation refer to https://github.com/octopus-network/ibc-go/blob/e40cdec6a3413fb3c8ea2a7ccad5e363ecd5a695/modules/apps/transfer/keeper/relay.go#L358
pub fn refund_packet_token<Ctx>(
	ctx: &Ctx,
	packet: Packet,
	data: FungibleTokenPacketData,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	// FungibleTokenPacketData data = packet.data
	// prefix = "{packet.sourcePort}/{packet.sourceChannel}/"
	// // we are the source if the denomination is not prefixed
	// source = denomination.slice(0, len(prefix)) !== prefix
	// if source {
	//   // sender was source chain, unescrow tokens back to sender
	//   escrowAccount = channelEscrowAddresses[packet.srcChannel]
	//   bank.TransferCoins(escrowAccount, data.sender, data.denomination, data.amount)
	// } else {
	//   // receiver was source chain, mint vouchers back to sender
	//   bank.MintCoins(data.sender, denomination, data.amount)
	// }
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

	// hash, err := types.ParseHexHash(hexHash)
	// if err != nil {
	// 	return "", sdkerrors.Wrap(types.ErrInvalidDenomForTransfer, err.Error())
	// }

	// denomTrace, found := k.GetDenomTrace(ctx, hash)
	// if !found {
	// 	return "", sdkerrors.Wrap(types.ErrTraceNotFound, hexHash)
	// }

	// fullDenomPath := denomTrace.GetFullDenomPath()
	// return fullDenomPath, nil
	let full_denom_path = String::default();
	Ok(full_denom_path)
}
