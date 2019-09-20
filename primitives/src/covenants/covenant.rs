use extended_primitives::Buffer;
use handshake_encoding::{Decodable, DecodingError, Encodable};

use super::{
    BidCovenant, ClaimCovenant, OpenCovenant, RedeemCovenant, RegisterCovenant, RenewCovenant,
    RevealCovenant, UpdateCovenant, TransferCovenant, FinalizeCovenant, RevokeCovenant
};

/// A Handshake covenant, which is a method of changing name state on the chain.
#[derive(PartialEq, Clone, Debug)]
pub enum Covenant {
    None,
    Claim(ClaimCovenant),
    Bid(BidCovenant),
    Open(OpenCovenant),
    Reveal(RevealCovenant),
    Redeem(RedeemCovenant),
    Register(RegisterCovenant),
    Update(UpdateCovenant),
    Renew(RenewCovenant),
    Transfer(TransferCovenant),
    Finalize(FinalizeCovenant),
    Revoke(RevokeCovenant),
}

impl Covenant {
    pub fn is_name(&self) -> bool {
        match self {
            Covenant::None => false,
            _ => true,
        }
    }

    pub fn is_dustworthy(&self) -> bool {
        match self {
            Covenant::None => true,
            Covenant::Bid(_) => true,
            _ => false,
        }
    }

    pub fn is_linked(&self) -> bool {
        match self {
            Covenant::None => false,
            Covenant::Claim(_) => false,
            Covenant::Open(_) => false,
            Covenant::Bid(_) => false,
            _ => true,
        }
    }

    //Returns whether the Covenant is spendable or not.
    pub fn is_unspendable(&self) -> bool {
        match self {
            Covenant::Revoke(_) => true,
            _ => false,
        }
    }

    //Returns whether or not the Coin inside of the covenant is spendable.
    pub fn is_nonspendable(&self) -> bool {
        match self {
            Covenant::None => false,
            Covenant::Open(_) => false,
            Covenant::Redeem(_) => false,
            _ => true,
        }
    }
}

impl Decodable for Covenant {
    type Err = DecodingError;

    fn decode(buffer: &mut Buffer) -> Result<Self, Self::Err> {
        let covenant_type = buffer.read_u8()?;

        match covenant_type {
            0 => Ok(Covenant::None),
            1 => {
                let covenant = ClaimCovenant::decode(buffer)?;
                Ok(Covenant::Claim(covenant))
            }
            2 => {
                let covenant = OpenCovenant::decode(buffer)?;
                Ok(Covenant::Open(covenant))
            }
            3 => {
                let covenant = BidCovenant::decode(buffer)?;
                Ok(Covenant::Bid(covenant))
            }
            4 => {
                let covenant = RevealCovenant::decode(buffer)?;
                Ok(Covenant::Reveal(covenant))
            }
            5 => {
                let covenant = RedeemCovenant::decode(buffer)?;
                Ok(Covenant::Redeem(covenant))
            }
            6 => {
                let covenant = RegisterCovenant::decode(buffer)?;
                Ok(Covenant::Register(covenant))
            }
            7 => {
                let covenant = UpdateCovenant::decode(buffer)?;
                Ok(Covenant::Update(covenant))
            }
            8 => {
                let covenant = RenewCovenant::decode(buffer)?;
                Ok(Covenant::Renew(covenant))
            }
            9 => {
                let covenant = TransferCovenant::decode(buffer)?;
                Ok(Covenant::Transfer(covenant))
            }
            10 => {
                let covenant = FinalizeCovenant::decode(buffer)?;
                Ok(Covenant::Finalize(covenant))
            }
            11 => {
                let covenant = RevokeCovenant::decode(buffer)?;
                Ok(Covenant::Revoke(covenant))
            }
            _ => Err(DecodingError::InvalidData(
                "Unknown Covenant Type".to_owned(),
            )),
        }
    }
}

impl Encodable for Covenant {
    fn size(&self) -> usize {
        match self {
            Covenant::None => 0,
            Covenant::Claim(claim) => claim.size(),
            Covenant::Open(open) => open.size(),
            Covenant::Bid(bid) => bid.size(),
            Covenant::Reveal(reveal) => reveal.size(),
            Covenant::Redeem(redeem) => redeem.size(),
            Covenant::Register(register) => register.size(),
            Covenant::Update(update) => update.size(),
            Covenant::Renew(renew) => renew.size(),
            Covenant::Transfer(transfer) => transfer.size(),
            Covenant::Finalize(finalize) => finalize.size(),
            Covenant::Revoke(revoke) => revoke.size(),
        }
    }

    fn encode(&self) -> Buffer {
        match self {
            Covenant::None => Buffer::new(),
            Covenant::Claim(claim) => claim.encode(),
            Covenant::Open(open) => open.encode(),
            Covenant::Bid(bid) => bid.encode(),
            Covenant::Reveal(reveal) => reveal.encode(),
            Covenant::Redeem(redeem) => redeem.encode(),
            Covenant::Register(register) => register.encode(),
            Covenant::Update(update) => update.encode(),
            Covenant::Renew(renew) => renew.encode(),
            Covenant::Transfer(transfer) => transfer.encode(),
            Covenant::Finalize(finalize) => finalize.encode(),
            Covenant::Revoke(revoke) => revoke.encode(),
        }
    }
}

//TODO finish up testing the global functions.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covenant_is_name() {
        let empty_cov = Covenant::None;

        assert!(!empty_cov.is_name());

        let bid = BidCovenant {
            name_hash: Default::default(),
            height: 0,
            name: Default::default(),
            hash: Default::default(),
        };

        let cov = Covenant::Bid(bid);

        assert!(cov.is_name());
    }
}
