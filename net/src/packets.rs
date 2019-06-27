use crate::common::USER_AGENT;
use crate::net_address::NetAddress;
use crate::types::{ProtocolVersion, Services};
use crate::Result;
use chrono::{DateTime, TimeZone, Utc};
use extended_primitives::Buffer;
use handshake_protocol::encoding::{Decodable, Encodable};
use handshake_protocol::network::Network;
use rand::Rng;

pub enum Packet {
    Version(VersionPacket),
    Verack,
}

impl Packet {
    pub fn parse(mut packet: Buffer) -> Result<(Buffer, u8)> {
        let magic = packet.read_u32()?;
        let _type = packet.read_u8()?;
        let size = packet.read_u32()?;

        //Check magic number, throw packet invalid magic number
        //Check size, and ensure it's below constant max message size. -> We already have
        //This checked in Brontide, but I think let's check it again here.

        Ok((packet, _type))
    }

    pub fn decode(packet: Buffer) -> Result<Self> {
        let (raw_packet, packet_type) = Packet::parse(packet)?;
        match packet_type {
            1 => {
                let packet = VersionPacket::decode(raw_packet)?;
                Ok(Packet::Version(packet))
            }
            _ => Ok(Packet::Verack),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PacketType {
    Version = 0,
    Verack = 1,
    Ping = 2,
    Pong = 3,
    GetAddr = 4,
    Addr = 5,
    Inv = 6,
    GetData = 7,
    NotFound = 8,
    GetBlocks = 9,
    GetHeaders = 10,
    Headers = 11,
    SendHeaders = 12,
    Block = 13,
    Tx = 14,
    Reject = 15,
    Mempool = 16,
    FilterLoad = 17,
    FilterAdd = 18,
    FilterClear = 19,
    MerkleBlock = 20,
    FeeFilter = 21,
    SendCompact = 22,
    CompactBlock = 23,
    GetBlockTransaction = 24,
    BlockTransaction = 25,
    GetProof = 26,
    Proof = 27,
    Claim = 28,
    Airdrop = 29,
    Unknown = 30,
    Internal = 31,
    Data = 32,
}

//Optionally all packets go inside of a Packet enum and we use that to implement frame, which we
//can remove from peer. Not sure if that's the way we want to go, but this should work for now.

#[derive(Clone, Debug)]
pub struct VersionPacket {
    _type: PacketType,
    version: ProtocolVersion,
    services: Services,
    //Check on this.
    time: DateTime<Utc>,
    remote: NetAddress,
    //This doesn't feel correct, probably should be a setBuffer TODO
    nonce: Buffer,
    agent: String,
    height: u32,
    no_relay: bool,
}

//Make Packet a trait, and have it include functions like size and encode.
impl VersionPacket {
    pub fn new(addr: NetAddress, height: u32, no_relay: bool) -> Self {
        //TODO we probably want to implement noncelist here.
        let nonce = rand::thread_rng().gen::<[u8; 8]>();
        VersionPacket {
            _type: PacketType::Version,
            version: ProtocolVersion::default(),
            services: Services::LOCAL_SERVICES,
            time: Utc::now(),
            remote: addr,
            agent: USER_AGENT.to_owned(),
            nonce: Buffer::from(nonce.to_vec()),
            height,
            no_relay,
        }
    }

    //Function to frame the version packet.
    //we should maybe make this a trait.
    //Trait Packet
    //should impl frame, and parse
    pub fn frame(&self, network: Network) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(network.magic());
        buffer.write_u8(self._type as u8);
        buffer.write_u32(self.size());

        //Write encoded packet
        buffer.extend(self.encode());

        buffer
    }

    fn decode(mut packet: Buffer) -> Result<Self> {
        let version = packet.read_u32()?;
        let services = packet.read_u32()?;
        packet.read_u32()?;
        let timestamp = packet.read_u64()?;
        let remote = NetAddress::decode(&mut packet)?;
        let nonce = packet.read_bytes(8)?;
        let agent_length = packet.read_u8()?;
        let agent = packet.read_string(agent_length as usize)?;
        let height = packet.read_u32()?;
        let no_relay = packet.read_u8()?;

        Ok(VersionPacket {
            _type: PacketType::Version,
            version: ProtocolVersion::from(version),
            services: Services::from_bits_truncate(services),
            time: Utc.timestamp(timestamp as i64, 0),
            remote,
            agent,
            nonce: Buffer::from(nonce),
            height,
            no_relay: no_relay == 1,
        })
    }
}

impl Encodable for VersionPacket {
    fn size(&self) -> u32 {
        let mut size = 0;
        size += 20;
        size += self.remote.size();
        size += 8;
        size += 1;
        size += self.agent.len() as u32;
        size += 5;
        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(self.version.as_u32()); //TODO protocol version should deref to a u32
        buffer.write_u32(self.services.bits());
        buffer.write_u32(0);
        buffer.write_u64(self.time.timestamp() as u64);
        buffer.extend(self.remote.encode());
        buffer.extend(self.nonce.clone());
        buffer.write_u8(self.agent.len() as u8);
        buffer.write_str(&self.agent);
        buffer.write_u32(self.height);
        buffer.write_u8(self.no_relay as u8);

        buffer
    }
}
