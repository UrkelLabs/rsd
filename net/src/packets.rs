use crate::common::USER_AGENT;
use crate::net_address::NetAddress;
use crate::types::{ProtocolVersion, Services};
use chrono::{DateTime, Utc};
use handshake_protocol::encoding::Encodable;
use handshake_types::Buffer;

#[derive(Copy, Clone)]
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

pub struct VersionPacket {
    _type: PacketType,
    version: ProtocolVersion,
    services: Services,
    //Check on this.
    time: DateTime<Utc>,
    remote: NetAddress,
    // nonce:
    agent: String,
    height: u32,
    no_relay: bool,
}

//Make Packet a trait, and have it include functions like size and encode.
impl VersionPacket {
    pub fn new(addr: NetAddress, height: u32, no_relay: bool) -> Self {
        VersionPacket {
            _type: PacketType::Version,
            version: ProtocolVersion::default(),
            services: Services::LocalServices,
            time: Utc::now(),
            remote: addr,
            agent: USER_AGENT.to_owned(),
            height,
            no_relay,
        }
    }

    //TODO implement
    //TODO maybe make this a fn rather than a method
    pub fn size(&self) -> u32 {
        0
    }

    pub fn encode(&self) {
        let mut buffer = Buffer::new();

        //Framing
        //Write magic number here TODO
        buffer.write_u32(0);
        buffer.write_u8(self._type as u8);
        buffer.write_u32(self.size());

        //Write remaning packet
        buffer.write_u32(self.version.as_u32()); //TODO protocol version should deref to a u32
        buffer.write_u32(self.services.value()); //TODO services should return u32 not u64
        buffer.write_u32(0);
        buffer.write_u64(self.time.timestamp() as u64);
        buffer.extend(self.remote.encode());
        //Need to write the remote here. TODO probably impl an encode function for PeerAddr
    }

    // bw.writeU32(this.version);
    // bw.writeU32(this.services);
    // bw.writeU32(0);
    // bw.writeU64(this.time);
    // this.remote.write(bw);
    // bw.writeBytes(this.nonce);
    // bw.writeU8(this.agent.length);
    // bw.writeString(this.agent, 'ascii');
    // bw.writeU32(this.height);
    // bw.writeU8(this.noRelay ? 1 : 0);

    // assert((cmd & 0xff) === cmd);
    // assert(Buffer.isBuffer(payload));
    // assert(payload.length <= 0xffffffff);

    // const msg = Buffer.allocUnsafe(9 + payload.length);

    // // Magic value
    // msg.writeUInt32LE(this.network.magic, 0, true);

    // // Command
    // msg[4] = cmd;

    // // Payload length
    // msg.writeUInt32LE(payload.length, 5, true);

    // payload.copy(msg, 9);

    // return msg;

    pub fn frame() {}
}

// /**
//  * Get serialization size.
//  * @returns {Number}
//  */
// getSize() {
//   let size = 0;
//   size += 20;
//   size += this.remote.getSize();
//   size += 8;
//   size += 1;
//   size += this.agent.length;
//   size += 5;
//   return size;
// }

// /**
//  * Write version packet to buffer writer.
//  * @param {BufferWriter} bw
//  */
// write(bw) {
//   bw.writeU32(this.version);
//   bw.writeU32(this.services);
//   bw.writeU32(0);
//   bw.writeU64(this.time);
//   this.remote.write(bw);
//   bw.writeBytes(this.nonce);
//   bw.writeU8(this.agent.length);
//   bw.writeString(this.agent, 'ascii');
//   bw.writeU32(this.height);
//   bw.writeU8(this.noRelay ? 1 : 0);
//   return this;
// }

// /**
//  * Inject properties from buffer reader.
//  * @private
//  * @param {BufferReader} br
//  */
// read(br) {
//   this.version = br.readU32();
//   this.services = br.readU32();

//   // Note: hi service bits
//   // are currently unused.
//   br.readU32();

//   this.time = br.readU64();
//   this.remote.read(br);
//   this.nonce = br.readBytes(8);
//   this.agent = br.readString(br.readU8(), 'ascii');
//   this.height = br.readU32();
//   this.noRelay = br.readU8() === 1;

//   return this;
// }
// }
