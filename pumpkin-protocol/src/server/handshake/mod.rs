use bytes::Buf;
use pumpkin_data::packet::serverbound::HANDSHAKE_INTENTION;
use pumpkin_macros::server_packet;

use crate::{
    ConnectionState, ServerPacket, VarInt,
    bytebuf::{ByteBuf, ReadingError},
};

#[server_packet(HANDSHAKE_INTENTION)]
pub struct SHandShake {
    pub protocol_version: VarInt,
    pub server_address: String, // 255
    pub server_port: u16,
    pub next_state: ConnectionState,
}

impl ServerPacket for SHandShake {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            protocol_version: bytebuf.try_get_var_int()?,
            server_address: bytebuf.try_get_string_len(255)?,
            server_port: bytebuf.try_get_u16()?,
            next_state: bytebuf
                .try_get_var_int()?
                .try_into()
                .map_err(|_| ReadingError::Message("Invalid Status".to_string()))?,
        })
    }
}
