use crate::common::{MAX_INV, USER_AGENT};
use crate::net_address::NetAddress;
use crate::types::{ProtocolVersion, Services};
use crate::Result;
use chrono::{DateTime, TimeZone, Utc};
use extended_primitives::Buffer;
use extended_primitives::Hash;
use extended_primitives::Uint256;
use extended_primitives::VarInt;
use handshake_primitives::Inventory;
use handshake_protocol::encoding::{Decodable, Encodable};
use handshake_protocol::network::Network;
use rand::Rng;

//TODO I think we might be able to remove packet types from all of these things, but for now keep
//them.
#[derive(Debug, PartialEq)]
pub enum Packet {
    Version(VersionPacket),
    Verack,
    Ping(PingPacket),
    Pong(PongPacket),
    GetAddr,
    Addr(AddrPacket),
    Inv(InvPacket),
    GetData,
    NotFound,
    GetBlocks(GetBlocksPacket),

    Unknown(UnknownPacket),
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

    //Function to frame the version packet.
    pub fn frame(&self, network: Network) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_u32(network.magic());
        buffer.write_u8(self.packet_type());
        buffer.write_u32(self.size());

        //Write encoded packet
        buffer.extend(self.encode());

        buffer
    }

    pub fn decode(packet: Buffer) -> Result<Self> {
        let (raw_packet, packet_type) = Packet::parse(packet)?;
        match packet_type {
            0 => {
                let packet = VersionPacket::decode(raw_packet)?;
                Ok(Packet::Version(packet))
            }
            1 => Ok(Packet::Verack),
            2 => {
                let packet = PingPacket::decode(raw_packet)?;
                Ok(Packet::Ping(packet))
            }
            3 => {
                let packet = PongPacket::decode(raw_packet)?;
                Ok(Packet::Pong(packet))
            }
            4 => Ok(Packet::GetAddr),
            5 => {
                let packet = AddrPacket::decode(raw_packet)?;
                Ok(Packet::Addr(packet))
            }
            6 => {
                let packet = InvPacket::decode(raw_packet)?;
                Ok(Packet::Inv(packet))
            }
            7 => Ok(Packet::GetData),
            8 => Ok(Packet::NotFound),
            9 => {
                let packet = GetBlocksPacket::decode(raw_packet)?;
                Ok(Packet::GetBlocks(packet))
            }
            _ => {
                let packet = UnknownPacket::decode(raw_packet)?;
                Ok(Packet::Unknown(packet))
            }
        }
    }

    //TODO maybe switch this to the trait encodable -> TODO
    pub fn encode(&self) -> Buffer {
        match self {
            Packet::Version(version) => version.encode(),
            //TODO check verack encoding.
            Packet::Verack => Buffer::new(),
            Packet::Ping(ping) => ping.encode(),
            Packet::Pong(pong) => pong.encode(),
            Packet::GetAddr => Buffer::new(),
            Packet::Addr(addr) => addr.encode(),
            Packet::Inv(inv) => inv.encode(),
            Packet::GetData => Buffer::new(),
            Packet::NotFound => Buffer::new(),
            Packet::GetBlocks(blocks) => blocks.encode(),
            _ => Buffer::new(),
        }
    }

    pub fn packet_type(&self) -> u8 {
        match self {
            Packet::Version(_) => 0,
            Packet::Verack => 1,
            _ => 2,
        }
    }

    pub fn size(&self) -> u32 {
        match self {
            Packet::Version(version) => version.size(),
            //TODO is verack size 0?
            Packet::Verack => 0,
            _ => 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct PingPacket {
    _type: PacketType,
    //TODO probably make this a custom type. -> I think it's the same nonce as hostname.
    nonce: Uint256,
}

impl PingPacket {
    pub fn decode(mut packet: Buffer) -> Result<Self> {
        //TODO
        // let nonce = packet.read_bytes(8)?;
        // let nonce = packet.read_u256()?;

        Ok(PingPacket {
            _type: PacketType::Ping,
            nonce: Default::default(),
        })
    }
}

impl Encodable for PingPacket {
    fn size(&self) -> u32 {
        8
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        //TOD switch when we fix nonce.
        // buffer.write_bytes(self.nonce);
        buffer.write_u256(self.nonce);

        buffer
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct PongPacket {
    _type: PacketType,
    //TODO probably make this a custom type. -> I think it's the same nonce as hostname.
    nonce: Uint256,
}

impl PongPacket {
    pub fn decode(mut packet: Buffer) -> Result<Self> {
        //TODO
        // let nonce = packet.read_bytes(8)?;
        // let nonce = packet.read_u256()?;

        Ok(PongPacket {
            _type: PacketType::Ping,
            nonce: Default::default(),
        })
    }
}

impl Encodable for PongPacket {
    fn size(&self) -> u32 {
        8
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        //TOD switch when we fix nonce.
        // buffer.write_bytes(self.nonce);
        buffer.write_u256(self.nonce);

        buffer
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AddrPacket {
    _type: PacketType,
    items: Vec<NetAddress>,
}

impl AddrPacket {
    pub fn decode(mut packet: Buffer) -> Result<Self> {
        let count = packet.read_varint()?;
        //TODO would it be faster to initalize with capacity here? since we know the count.
        let mut items = Vec::new();
        for _ in 0..count.to_u64() {
            items.push(NetAddress::decode(&mut packet)?);
        }

        Ok(AddrPacket {
            _type: PacketType::Addr,
            items,
        })
    }
}

impl Encodable for AddrPacket {
    fn size(&self) -> u32 {
        let mut size = 0;
        let length = VarInt::from(self.items.len() as u64);
        size += length.encoded_size();
        let items = self.items.iter();
        for addr in items {
            size += addr.size();
        }

        size
    }

    fn encode(&self) -> Buffer {
        let mut buffer = Buffer::new();

        buffer.write_varint(self.items.len());
        let items = self.items.iter();
        for item in items {
            buffer.extend(item.encode());
        }

        buffer
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InvPacket {
    _type: PacketType,
    items: Vec<Inventory>,
}

impl InvPacket {
    pub fn decode(mut packet: Buffer) -> Result<Self> {
        let count = packet.read_varint()?;

        let mut items = Vec::new();
        for _ in 0..count.to_u64() {
            items.push(Inventory::decode(&mut packet)?);
        }

        Ok(InvPacket {
            _type: PacketType::Inv,
            items,
        })
    }
}

impl Encodable for InvPacket {
    fn size(&self) -> u32 {
        let mut size = 0;
        let length = VarInt::from(self.items.len() as u64);
        size += length.encoded_size();

        let items = self.items.iter();
        for item in items {
            size += item.size();
        }

        size
    }

    fn encode(&self) -> Buffer {
        assert!(self.items.len() < MAX_INV as usize);

        let mut buffer = Buffer::new();

        buffer.write_varint(self.items.len());

        let items = self.items.iter();

        for item in items {
            buffer.extend(item.encode());
        }

        buffer
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GetBlocksPacket {
    _type: PacketType,
    locator: Vec<Hash>,
    stop: Hash,
}

impl GetBlocksPacket {
    pub fn decode(mut packet: Buffer) -> Result<Self> {
        let count = packet.read_varint()?;

        //TODO probably catch this error, and destroy the peer.
        //TODO have count.to_usize, count.to_u32, etc
        assert!(count.as_u64() <= MAX_INV as u64);

        let mut locator: Vec<Hash> = Vec::new();

        for _ in 0..count.to_u64() {
            locator.push(packet.read_hash()?);
        }

        let stop = packet.read_hash()?;

        Ok(GetBlocksPacket {
            _type: PacketType::GetBlocks,
            locator,
            stop,
        })
    }
}

impl Encodable for GetBlocksPacket {
    fn size(&self) -> u32 {
        let mut size = 0;
        let length = VarInt::from(self.locator.len() as u64);
        size += length.encoded_size();
        //Each hash is 32 bytes.
        size += self.locator.len() as u32 * 32;
        //Stop size
        size += 32;
        size
    }

    fn encode(&self) -> Buffer {
        assert!(self.locator.len() < MAX_INV as usize);

        let mut buffer = Buffer::new();

        buffer.write_varint(self.locator.len());
        let items = self.locator.iter();
        for item in items {
            buffer.write_hash(*item);
        }

        buffer.write_hash(self.stop);

        buffer
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnknownPacket {
    _type: PacketType,
    data: Buffer,
}

impl UnknownPacket {
    pub fn decode(mut packet: Buffer) -> Result<Self> {
        Ok(UnknownPacket {
            _type: PacketType::Unknown,
            data: packet,
        })
    }
}

impl Encodable for UnknownPacket {
    fn size(&self) -> u32 {
        self.data.len() as u32
    }

    fn encode(&self) -> Buffer {
        self.data.clone()
    }
}
