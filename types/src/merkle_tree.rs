use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use extended_primitives::Hash;
use hex::FromHex;

//Merkle tree type for use in Handshake
//@todo
//@todo can I generalize this entire thing?
#[derive(Debug)]
pub struct MerkleTree {
    steps: Vec<Hash>,
}

impl MerkleTree {
    pub fn from_leaves(leaves: Vec<Hash>) -> Self {
        let mut nodes = Vec::new();
        // let mut steps = Vec::new();

        let mut output = [0; 32];
        let mut sh = Blake2b::new(32);
        let bytes = [0; 32];
        sh.input(&bytes);
        sh.result(&mut output);
        let sentinel = Hash::from(output);

        for hash in leaves.iter() {
            let mut sh = Blake2b::new(32);
            let mut output = [0; 32];
            sh.input(&[0x00]);
            sh.input(&hash.to_array());
            sh.result(&mut output);
            let leaf = Hash::from(output);
            nodes.push(leaf);
        }

        let mut len = nodes.len();
        let mut i = 0;

        if len == 0 {
            nodes.push(sentinel);
            return MerkleTree { steps: nodes };
        }

        // len = 3;
        while len > 1 {
            for j in (0..len).step_by(2) {
                let l = j;
                let r = j + 1;

                let left = nodes[i + l];

                let right = if r < len { nodes[i + r] } else { sentinel };

                let mut sh = Blake2b::new(32);
                let mut output = [0; 32];
                sh.input(&[0x01]);
                sh.input(&left.to_array());
                sh.input(&right.to_array());
                sh.result(&mut output);

                nodes.push(Hash::from(output));
            }

            i += len;

            //@todo review
            // len = (len + 1) >>> 1;
            len /= 2;
        }

        MerkleTree { steps: nodes }
    }

    pub fn get_root(&self) -> Hash {
        self.steps[self.steps.len() - 1]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merkle_tree_generation() {
        let leaves = vec![
            Hash::from("6ccc9037d0be8b70207fd3e384565743d3925e27e0923a57fa7fc51f8e951ba9"),
            Hash::from("fa43aa977aa4f3e4ff8bbd8c05fa81eb0d00320bfca60f0acb129e8e696d99cf"),
        ];

        let tree = MerkleTree::from_leaves(leaves);

        dbg!(tree.get_root());
    }
}

//@todo JSON serialization.
//@todo binary serialization.
//@todo testing
//@todo abstract core functionality away from here -> maybe a new crate. Then we can benchmark much
//better
//@todo helper functions and defaults.
