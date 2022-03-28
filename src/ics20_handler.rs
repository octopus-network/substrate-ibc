use super::*;

use crate::alloc::string::ToString;
use event::primitive::{
	ChannelId, ClientId, ClientState as EventClientState, ClientType, ConnectionId, Height,
	Packet as IbcPacket, PortId, Timestamp,
};
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as Ics20Error,
	},
	core::ics04_channel::packet::Packet,
};
use ibc_proto::ibc::apps::transfer::v2::FungibleTokenPacketData as IBCFungibleTokenPacketData;

use frame_support::{
	sp_runtime::traits::{AtLeast32BitUnsigned, CheckedConversion},
	sp_std::fmt::Debug,
	traits::{tokens::fungibles, Currency, ExistenceRequirement::AllowDeath},
	PalletId,
};
use sp_runtime::traits::AccountIdConversion;

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
		let ret = T::Currency::transfer(&sender, &escrow_account, amount, AllowDeath).unwrap(); // TODO: unwrap()
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
			Err(_err) =>
				ack = FungibleTokenPacketAcknowledgement::Err(FungibleTokenPacketError {
					error: "transfer coin failed".to_string(),
				}),
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
			Err(_) =>
				ack = FungibleTokenPacketAcknowledgement::Err(FungibleTokenPacketError {
					error: "mint coins failed".to_string(),
				}),
		}
	}

	Ok(ack)
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
	acknowledgement: Vec<u8>,
) -> Result<(), Ics20Error>
where
	Ctx: Ics20Context,
{
	let acknowledgement = String::from_utf8(acknowledgement).unwrap();
	let acknowledgement: FungibleTokenPacketAcknowledgement =
		serde_json::from_str(&acknowledgement).unwrap();
	// if the transfer failed, refund the token
	match acknowledgement {
		FungibleTokenPacketAcknowledgement::Err(_ack) => {
			return refund_packet_token(ctx, packet, data)
		},
		_ => unimplemented!(),
	}

	Ok(())
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
	// the packet timeout_out, so refund the tokens
	let ret = refund_packet_token(ctx, packet, data);

	Ok(())
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
