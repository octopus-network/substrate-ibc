use super::*;
use crate::ics20_handler;
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as Ics20Error,
	},
	core::{
		ics04_channel::{channel::Counterparty, channel::Order, packet::Packet, Version},
		ics05_port::capabilities::Capability,
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::ibc_module::IBCModule,
	},
	signer::Signer,
};
use ibc_proto::ibc::apps::transfer::v2::FungibleTokenPacketData;
use ibc_proto::ibc::core::channel::v1::Acknowledgement;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ics20IBCModule;

impl IBCModule for Ics20IBCModule {
	// OnChanOpenInit implements the IBCModule interface
	// refter to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L64
	fn on_chan_open_init<Ctx>(
		&self,
		ctx: &Ctx,
		order: Order,
		connection_hops: Vec<ConnectionId>,
		port_id: PortId,
		channel_id: ChannelId,
		channel_cap: &Capability,
		counterparty: Counterparty,
		version: Version,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		// if err := ValidateTransferChannelParams(ctx, im.keeper, order, portID, channelID); err !=
		// nil {     return err
		// }

		// if version != types.Version {
		//     return sdkerrors.Wrapf(types.ErrInvalidVersion, "got %s, expected %s", version,
		// types.Version) }

		// // Claim channel capability passed back by IBC module
		// if err := im.keeper.ClaimCapability(ctx, chanCap, host.ChannelCapabilityPath(portID,
		// channelID)); err != nil {     return err
		// }

		// return nil
		Ok(())
	}

	// OnChanOpenTry implements the IBCModule interface.
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L91
	fn on_chan_open_try<Ctx>(
		&self,
		ctx: &Ctx,
		order: Order,
		connection_hops: Vec<ConnectionId>,
		port_id: PortId,
		channel_id: ChannelId,
		channel_cap: &Capability,
		counterparty: Counterparty,
		counterparty_version: Version,
	) -> Result<Version, Ics20Error>
	where
		Ctx: Ics20Context,
	{
		// if err := ValidateTransferChannelParams(ctx, im.keeper, order, portID, channelID); err !=
		// nil {     return "", err
		// }

		// if counterpartyVersion != types.Version {
		//     return "", sdkerrors.Wrapf(types.ErrInvalidVersion, "invalid counterparty version:
		// got: %s, expected %s", counterpartyVersion, types.Version) }

		// // Module may have already claimed capability in OnChanOpenInit in the case of crossing
		// hellos // (ie chainA and chainB both call ChanOpenInit before one of them calls
		// ChanOpenTry) // If module can already authenticate the capability then module already
		// owns it so we don't need to claim // Otherwise, module does not have channel capability
		// and we must claim it from IBC if !im.keeper.AuthenticateCapability(ctx, chanCap,
		// host.ChannelCapabilityPath(portID, channelID)) {     // Only claim channel capability
		// passed back by IBC module if we do not already own it     if err :=
		// im.keeper.ClaimCapability(ctx, chanCap, host.ChannelCapabilityPath(portID, channelID));
		// err != nil {         return "", err
		//     }
		// }

		// return types.Version, nil
		let version = Version::ics20();
		Ok(version)
	}

	// OnChanOpenAck implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L124
	fn on_chan_open_ack<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
		counterparty_version: Version,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		// if counterpartyVersion != types.Version {
		//     return sdkerrors.Wrapf(types.ErrInvalidVersion, "invalid counterparty version: %s,
		// expected %s", counterpartyVersion, types.Version) }
		// return nil
		Ok(())
	}

	// OnChanOpenConfirm implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L137
	fn on_chan_open_confirm<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		Ok(())
	}
	// OnChanCloseInit implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L146
	fn on_chan_close_init<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		// Disallow user-initiated channel closing for transfer channels
		// return sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, "user cannot close channel")
		Ok(())
	}
	// OnChanCloseConfirm implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L156
	fn on_chan_close_confirm<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		Ok(())
	}
	// OnRecvPacket implements the IBCModule interface. A successful acknowledgement
	// is returned if the packet data is succesfully decoded and the receive application
	// logic returns without error.
	// refer to ibc-go https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L167
	fn on_recv_packet<Ctx>(
		&self,
		ctx: &Ctx,
		packet: Packet,
		relayer: Signer,
	) -> Result<Vec<u8>, Ics20Error>
	where
		Ctx: Ics20Context,
	{
		// 	ack := channeltypes.NewResultAcknowledgement([]byte{byte(1)})

		// var data types.FungibleTokenPacketData
		// if err := types.ModuleCdc.UnmarshalJSON(packet.GetData(), &data); err != nil {
		// 	ack = channeltypes.NewErrorAcknowledgement("cannot unmarshal ICS-20 transfer packet data")
		// }

		// // only attempt the application logic if the packet data
		// // was successfully decoded
		// if ack.Success() {
		// 	err := im.keeper.OnRecvPacket(ctx, packet, data)
		// 	if err != nil {
		// 		ack = types.NewErrorAcknowledgement(err)
		// 	}
		// }

		let ack_value = vec![1];

		//TODO: build FungibleTokenPacketData
		// let data = FungibleTokenPacketData::decode(&mut &packet.data[..]).unwrap();

		// TODO: handle recv packet
		//let result = ics20_handler::handle_recv_packet(ctx, packet, data)

		//TODO:
		// if result is err
		//let ack_result = err.to_string.get_bytes
		// if result is ok let ack_value = vec![1];

		let ack_result = vec![1];
		let ack = Acknowledgement::encode(&ack_result);
		Ok(ack)
	}

	// OnAcknowledgementPacket implements the IBCModule interface
	// refer to ibc-go https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L204
	fn on_acknowledgement_packet<Ctx>(
		&self,
		ctx: &Ctx,
		packet: Packet,
		acknowledgement: Vec<u8>,
		relayer: Signer,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		//TODO: build FungibleTokenPacketData
		// var data types.FungibleTokenPacketData
		// if err := types.ModuleCdc.UnmarshalJSON(packet.GetData(), &data); err != nil {
		// 	return sdkerrors.Wrapf(sdkerrors.ErrUnknownRequest, "cannot unmarshal ICS-20 transfer
		// packet data: %s", err.Error()) }

		// let data = FungibleTokenPacketData::decode(&mut &packet.data[..]).unwrap();

		// TODO: handle ack packet
		// if err := im.keeper.OnAcknowledgementPacket(ctx, packet, data, ack); err != nil {
		// 	return err
		// }
		//let ack = Acknowledgement::decode(&mut &acknowledgement[..]).unwrap();
		//ics20_handler::handle_ack_packet(ctx, packet, data, ack)
		Ok(())
	}

	// OnTimeoutPacket implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L204
	fn on_timeout_packet<Ctx>(
		&self,
		ctx: &Ctx,
		packet: Packet,
		relayer: Signer,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		//TODO: build FungibleTokenPacketData
		// let data = FungibleTokenPacketData::decode(&mut &packet.data[..]).unwrap();

		// TODO: handle ack packet/refund tokens
		//ics20_handler::handle_timeout_packet(ctx, packet, data)

		Ok(())
	}
}
