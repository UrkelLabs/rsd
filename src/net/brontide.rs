use chacha20_poly1305_aead;
use hkdf::Hkdf;
use rand::rngs::OsRng;
use secp256k1::ecdh::SharedSecret;
use secp256k1::rand;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

use crate::Buffer;

//TODO move to common file once moved to new repo.
//TODO we need to ensure the Option<> on tag is correct
const ROTATION_INTERVAL: u32 = 1000;
const PROTOCOL_NAME: &str = "Noise_XK_secp256k1_ChaChaPoly_SHA256";
const PROLOGUE: &str = "hns";
//Double check this type... I think it needs to just be 1 byte, but can't be sure. TODO
const VERSION: u8 = 0;
// pub struct Key([u8; 32]);
//
// TODO need to reimplement all Buffers as their own type most likely.

//TODO does this need to be public?
struct CipherState {
    //Double check on the size here TODO
    nonce: u32, //96 bit?
    secret_key: Buffer,
    salt: Buffer,
    iv: Buffer, //Capped at 12 bytes though, so maybe we need a new type.
}

//TODO again check on whether these things need to be public or not.
impl CipherState {
    pub fn update(&mut self) -> Buffer {
        //Refer to above TODO
        self.iv.write_u32(self.nonce);
        self.iv
    }

    //Todo maybe this a new function.
    pub fn init_key(&mut self, key: Buffer) {
        self.secret_key = key;
        self.nonce = 0;
        self.update();
    }

    //New with salt
    pub fn init_salt(&mut self, key: Buffer, salt: Buffer) {
        self.salt = salt;
        self.init_key(key);
    }

    pub fn rotate_key(&mut self) {
        let info = Buffer::new();
        let old = self.secret_key;
        let (salt, next) = expand(old, self.salt, info);

        self.salt = Buffer::from(salt);
        self.init_key(Buffer::from(next))
    }

    //TODO this needs heavy testing.
    pub fn encrypt(&mut self, pt: Buffer, ad: Buffer) -> Buffer {
        let mut ciphertext = Vec::with_capacity(pt.len());

        //TODO can't unwrap, need actual error handling here
        let tag =
            chacha20_poly1305_aead::encrypt(&self.secret_key, &self.iv, &ad, &pt, &mut ciphertext)
                .unwrap();

        self.nonce += 1;
        self.update();

        if self.nonce == ROTATION_INTERVAL {
            self.rotate_key();
        }

        Buffer::from(tag.to_vec())
    }

    pub fn decrypt(&mut self, ct: Buffer, tag: Buffer, ad: Buffer) -> bool {
        let mut plaintext = Vec::with_capacity(ct.len());

        let result = chacha20_poly1305_aead::decrypt(
            &self.secret_key,
            &self.iv,
            &ad,
            &tag,
            &ct,
            &mut plaintext,
        );

        match result {
            Err(_) => false,
            Ok(_) => {
                self.nonce += 1;
                self.update();

                if self.nonce == ROTATION_INTERVAL {
                    self.rotate_key();
                }

                true
            }
        }
    }
}

//TODO move these to the top, in the separate package move these to util.
fn expand(secret: Buffer, salt: Buffer, info: Buffer) -> (Vec<u8>, Vec<u8>) {
    //hk.prk
    let hk = Hkdf::<Sha256>::extract(Some(&salt), &secret);
    let mut out = [0u8; 64];
    //TODO remove unwrap
    hk.expand(&info, &mut out).unwrap();

    //TODO double check this
    (out[0..32].to_vec(), out[32..64].to_vec())
}

fn get_public_key(private_key: Buffer) -> Buffer {
    let secp = Secp256k1::new();
    //TODO handle this error correctly.
    let secret_key = SecretKey::from_slice(&private_key).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    Buffer::from(public_key.to_string())
}

//TODO this needs a lot of testing
fn ecdh(public_key: Buffer, private_key: Buffer) -> Buffer {
    //TODO super ugly, let's clean this up with better error handling
    let secret = SharedSecret::new(
        &PublicKey::from_slice(&public_key).unwrap(),
        &SecretKey::from_slice(&private_key).unwrap(),
    );

    //TODO this is how we use the FFI library better, use this example for the rest of the code.
    let secret_vec = secret[..].to_vec();

    let digest = Sha256::digest(&secret_vec);
    Buffer::from(digest.as_slice().to_vec())
}
// function ecdh(publicKey, privateKey) {
//   const secret = secp256k1.derive(publicKey, privateKey, true);
//   return sha256.digest(secret);
// }

pub struct SymmetricState {
    cipher: CipherState,
    chain: Buffer,  // chaining key
    temp: Buffer,   // temp key
    digest: Buffer, // handshake digest
}

impl SymmetricState {
    pub fn init_symmetric(&mut self, protocol_name: &str) {
        //I think this has to be a set size Buffer.
        let empty = Buffer::new();
        let proto = Buffer::from(protocol_name);

        let digest = Sha256::digest(&proto);
        self.digest = Buffer::from(digest.as_slice().to_vec());
        self.chain = self.digest;
        self.cipher.init_key(empty);
    }

    pub fn mix_key(&mut self, input: Buffer) {
        //I think this has to be a set size Buffer.
        let info = Buffer::new();
        let secret = input;
        let salt = self.chain;

        let (chain, temp) = expand(secret, salt, info);

        self.chain = Buffer::from(chain);
        self.temp = Buffer::from(temp);

        self.cipher.init_key(self.temp);
    }

    //TODO review
    pub fn mix_digest(&mut self, data: Buffer, tag: Option<Buffer>) -> Buffer {
        let mut hasher = Sha256::new();

        hasher.input(self.digest);
        hasher.input(data);
        if let Some(tag_ok) = tag {
            hasher.input(tag_ok);
        };

        let result = hasher.result();

        Buffer::from(result.as_slice().to_vec())
    }

    //TODO test if tag as an option handles this behavior correctly.
    pub fn mix_hash(&mut self, data: Buffer, tag: Option<Buffer>) {
        self.digest = self.mix_digest(data, tag);
    }

    //pt = plaintext, let's make that more verbose TODO so the code is more readable.
    pub fn encrypt_hash(&mut self, pt: Buffer) -> Buffer {
        let tag = self.cipher.encrypt(pt, self.digest);

        self.mix_hash(pt, Some(tag));

        tag
    }

    //ct == CipherText, make this more verbose as above TODO
    pub fn decrypt_hash(&mut self, ct: Buffer, tag: Buffer) -> bool {
        let digest = self.mix_digest(ct, Some(tag));

        let result = self.cipher.decrypt(ct, tag, self.digest);

        if result {
            self.digest = digest;
            true
        } else {
            false
        }
    }
}

pub struct HandshakeState {
    symmetric: SymmetricState,
    initiator: bool,
    local_static: Buffer,
    local_ephemeral: Buffer,
    remote_static: Buffer,
    remote_ephemeral: Buffer,
}

impl HandshakeState {
    pub fn generate_key() -> Buffer {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);
        Buffer::from(secret_key.to_string())
    }

    pub fn init_state(
        &self,
        initiator: bool,
        prologue: &str,
        local_pub: Buffer,
        remote_pub: Option<Buffer>,
    ) {
        let remote_public_key: Buffer;
        self.initiator = initiator;
        self.local_static = local_pub;
        if let Some(remote_pub_ok) = remote_pub {
            remote_public_key = remote_pub_ok
        } else {
            //Should be zero key not buffer new, TODO
            remote_public_key = Buffer::new()
        }

        self.remote_static = remote_public_key;

        self.symmetric.init_symmetric(PROTOCOL_NAME);
        //Might have to make sure this works as ascii TODO
        self.symmetric.mix_hash(Buffer::from(prologue), None);

        if initiator {
            //TODO we need to test this behavior, but I think the general idea is we want to mix
            //this with a zero hash buffer. so 32 bytes of 0s.
            self.symmetric.mix_hash(remote_public_key, None)
        } else {
            //Switch this with the get public function TODO
            let secp = Secp256k1::new();
            //TODO handle this error correctly.
            let secret_key =
                SecretKey::from_slice(&local_pub).expect("32 bytes, within curve order");
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);
            //TODO review this, not sure I trust converting the public key to string then reading
            //it in the buffer.
            self.symmetric
                .mix_hash(Buffer::from(public_key.to_string()), None);
        }
    }
}

//TODO let's review props in this struct
pub struct Brontide {
    handshake_state: HandshakeState,
    send_cipher: CipherState,
    receive_cipher: CipherState,
}

impl Brontide {
    //TODO review if this is option or not.
    pub fn init(&self, initiator: bool, local_pub: Buffer, remote_pub: Option<Buffer>) {
        self.handshake_state
            .init_state(initiator, PROLOGUE, local_pub, remote_pub);
    }

    pub fn gen_act_one(&self) -> Buffer {
        // e
        self.handshake_state.local_ephemeral = HandshakeState::generate_key();
        let ephemeral = get_public_key(self.handshake_state.local_ephemeral);
        //TODO double check this.
        self.handshake_state.symmetric.mix_hash(ephemeral, None);

        //ec
        let s = ecdh(
            self.handshake_state.remote_static,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(ephemeral);

        //TODO needs to be an empty buffer of 32 bytes. - Make this a constant when moved to new
        //package
        let tag = self.handshake_state.symmetric.encrypt_hash(Buffer::new());

        //const ACT_ONE_SIZE = 50;
        // let act_one = Buffer::new();
        let act_one = [0_u8; 50];
        act_one[0] = VERSION;
        //Double check this operation TODO
        //Might have to splice from 1..ephemeral.len() + 1
        act_one
            .to_vec()
            .splice(1..act_one.len(), ephemeral.into_iter());

        //Double check this operation TODO
        //Might have to splice from 1...tag.len() + 34
        act_one.to_vec().splice(34..act_one.len(), tag.into_iter());

        Buffer::from(act_one.to_vec())
    }

    //This is going to have to return a Result type to catch errors, TODO
    pub fn recv_act_one(&mut self, act_one: [u8; 50]) {
        if act_one[0] != VERSION {
            //throw error here TODO
            println!("Act one: bad version.");
        }

        //TODO check these operations to ensure proper slicing //inclusive/exclusive etc.
        //TODO also check on the borrowing here, doesn't smell right.
        let e = &act_one[1..34];
        let p = &act_one[34..act_one.len()];

        //We just want to verify here, might be an easier way than creating the actual key.
        //TODO
        let result = PublicKey::from_slice(e);

        if !result.is_ok() {
            //Throw error in here.
            println!("act one: bad key");
        }

        //e
        //TODO code smell
        self.handshake_state.remote_ephemeral = Buffer::from(e.to_owned());
        self.handshake_state
            .symmetric
            .mix_hash(self.handshake_state.remote_ephemeral, None);

        //es
        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_static,
        );
        self.handshake_state.symmetric.mix_key(s);

        //TODO must be empty buffer, not new buffer.
        //TODO code smell
        if !self
            .handshake_state
            .symmetric
            .decrypt_hash(Buffer::new(), Buffer::from(p.to_owned()))
        {
            //throw error
            println!("Act one: bad tag.");
        }
    }

    pub fn gen_act_two(&mut self) -> Buffer {
        // e
        self.handshake_state.local_ephemeral = HandshakeState::generate_key();

        let ephemeral = get_public_key(self.handshake_state.local_ephemeral);

        self.handshake_state.symmetric.mix_hash(ephemeral, None);

        // ee
        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(s);

        //TODO again this needs to be empty buffer, NOT new buffer.
        let tag = self.handshake_state.symmetric.encrypt_hash(Buffer::new());

        // const ACT_TWO_SIZE = 50;
        let act_two = [0_u8; 50];
        act_two[0] = VERSION;

        //TODO all the issues from act one apply here as well, this code needs to be thoroughly
        //checked and tested.
        act_two
            .to_vec()
            .splice(1..act_two.len(), ephemeral.into_iter());

        act_two.to_vec().splice(34..act_two.len(), tag.into_iter());

        Buffer::from(act_two.to_vec())
    }

    pub fn recv_act_two(&mut self, act_two: [u8; 50]) {
        if act_two[0] != VERSION {
            //throw error here TODO
            println!("Act two: bad version.");
        }

        //TODO check these operations to ensure proper slicing //inclusive/exclusive etc.
        //TODO also check on the borrowing here, doesn't smell right.
        let e = &act_two[1..34];
        let p = &act_two[34..act_two.len()];

        //We just want to verify here, might be an easier way than creating the actual key.
        //TODO
        let result = PublicKey::from_slice(e);

        if !result.is_ok() {
            //Throw error in here.
            println!("act one: bad key");
        }

        //e
        //TODO code smell
        self.handshake_state.remote_ephemeral = Buffer::from(e.to_owned());
        self.handshake_state
            .symmetric
            .mix_hash(self.handshake_state.remote_ephemeral, None);

        //es
        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(s);

        //TODO must be empty buffer, not new buffer.
        //TODO code smell
        if !self
            .handshake_state
            .symmetric
            .decrypt_hash(Buffer::new(), Buffer::from(p.to_owned()))
        {
            //throw error
            println!("Act two: bad tag.");
        }
    }

    pub fn gen_act_three(&mut self) -> Buffer {
        let our_pub_key = get_public_key(self.handshake_state.local_static);
        let tag_1 = self.handshake_state.symmetric.encrypt_hash(our_pub_key);
        let ct = our_pub_key;

        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_static,
        );
        self.handshake_state.symmetric.mix_key(s);

        //TODO again must be Buffer empty not new.
        let tag_2 = self.handshake_state.symmetric.encrypt_hash(Buffer::new());

        //const ACT_THREE_SIZE = 66;
        let act_three = [0_u8; 66];
        act_three[0] = VERSION;

        //TODO code smell
        act_three
            .to_vec()
            .splice(1..act_three.len(), ct.into_iter());
        act_three
            .to_vec()
            .splice(34..act_three.len(), tag_1.into_iter());
        act_three
            .to_vec()
            .splice(50..act_three.len(), tag_2.into_iter());

        self.split();

        Buffer::from(act_three.to_vec())
    }

    pub fn recv_act_three(&mut self, act_three: [u8; 66]) {
        if act_three[0] != VERSION {
            //Throw error in here
            println!("Act three: bad version");
        }

        //TODO code smell here...
        let s1 = Buffer::from(act_three[1..34].to_vec());
        let p1 = Buffer::from(act_three[34..50].to_vec());
        let s2 = Buffer::from(act_three[50..50].to_vec());
        let p2 = Buffer::from(act_three[50..66].to_vec());

        // s
        if self.handshake_state.symmetric.decrypt_hash(s1, p1) {
            //Throw error
            println!("act three: bad tag");
        }

        let remote_public = s1;

        let result = PublicKey::from_slice(&remote_public);

        if result.is_err() {
            //Throw error here TODO
            println!("act three: bad key.");
        }

        self.handshake_state.remote_static = remote_public;

        // se
        let se = ecdh(
            self.handshake_state.remote_static,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(se);

        if self.handshake_state.symmetric.decrypt_hash(s2, p2) {
            //Throw error, bad tag
            println!("act three bad tag");
        }

        self.split();
    }

    //TODO write and read
    //pub fn write(data: Buffer) {
    //    if data.len() <= 0xffff {
    //        //throw error -> Not sure what yet though TODO
    //    }

    //    //Needs to be a packet of length 2 + 16 + data.len() + 16
    //    //TODO I think this is correct
    //    let packet = Vec<u8>;

    //}

    //TODO review thoroughly AND TEST
    pub fn split(&mut self) {
        //TODO must be buffer empty not new
        let (h1, h2) = expand(
            Buffer::new(),
            self.handshake_state.symmetric.chain,
            Buffer::new(),
        );

        if self.handshake_state.initiator {
            let send_key = Buffer::from(h1);
            self.send_cipher
                .init_salt(send_key, self.handshake_state.symmetric.chain);
            let recv_key = Buffer::from(h2);
            self.receive_cipher
                .init_salt(recv_key, self.handshake_state.symmetric.chain);
        } else {
            let recv_key = Buffer::from(h1);
            self.receive_cipher
                .init_salt(recv_key, self.handshake_state.symmetric.chain);
            let send_key = Buffer::from(h2);
            self.send_cipher
                .init_salt(send_key, self.handshake_state.symmetric.chain);
        }
    }
}
