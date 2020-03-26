# Handshake Primitives

This modules houses all of the primitives that are used in Handshake. 

## Overview
TODO

## Building

Building the primitives module is similar to other modules in that it can be built with just a `cargo build`.
We also add some optional features in this library. Due to the amount of code bloat that is added with json encoding and decoding, we 
have feature gated that behind the feature `serde`.

To build this module with that feature enabled, run the following command: `cargo build --features=serde`. This will enabled JSON encoding and decoding for all primitives.

## Testing
Currently the only tests we have for primitives are unit tests. These are defined within the files themselves.

To run all unit tests, use the following command: `cargo test`.

## Organization
```
   primitives
      └── block_template
          └── airdrop.rs
          └── builder.rs
          └── json.rs
          └── mod.rs
      └── covenants
          └── bid.rs
          └── claim.rs
          └── covenant.rs
          └── finalize.rs
          └── mod.rs
          └── open.rs
          └── redeem.rs
          └── register.rs
          └── renew.rs
          └── reveal.rs
          └── revoke.rs
          └── transfer.rs
          └── update.rs
      └── transaction
          └── input.rs
          └── mod.rs
          └── outpoint.rs
          └── output.rs
          └── transaction.rs
      └── address.rs
      └── block.rs
      └── claim.rs
      └── headers.rs
      └── inventoy.rs
      └── lib.rs
```

## Roadmap

There are only a handful of primitives that are not yet implemented. They are listed below: 

- [ ] Airdrop Proof
- [ ] Airdrop Key
- [ ] Claim (Partial)
- [ ] Coin
- [ ] Key Ring

