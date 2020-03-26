# Handshake Rust Daemon

Warning - This is a highly experimental repository at its current state. I would not recommend using this library for consensus code until it has been thoroughly testing.

This library will likely drastically change over the coming weeks. 

## Organization

Currently RSD is structured into many modules, none of which produce a binary application that can be run. The main use of RSD currently is as libraries
imported by other projects. The goal is that when a sufficient amount of Handshake modules have been built to combine into a binary, we will build those at that time. We have done our best to mimic the structure that HSD follows. 

The current module organization can be seen below: 
```
   rsd 
          └── blockchain (Unfinished) # module containing the chain and it's functionality. 
          └── encoding   (Stable)     # houses the traits for encoding and decoding for primitives.
          └── mining     (Unfinished) # a cpu miner mainly used for testing.
          └── net        (Partial)    # handles all networking and p2p functions. 
          └── primitives (Partial)    # contains all Handshake primitives.
          └── protocol   (Partial)    # all consensus and network specific constants.
          └── script     (Partial)    # maintains the witness code.
          └── store      (Unfinished) # interface to the storage layer of the chain.
          └── types      (Partial)    # general types that are needed as building blocks of primitives.
```

## Development

To build a specific module, cd into that module and run `cargo build`. To test any module run `cargo test`.
