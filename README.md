# Template for a Hyle-Risc0 smart contract

This basic implementation is based on "counter" contract, that increment / decrement a value.

## Prerequisites

- [Install Rust](https://www.rust-lang.org/tools/install) (you'll need `rustup` and Cargo).
- For our example, [install RISC Zero](https://dev.risczero.com/api/zkvm/install).
- Run a local devnet node:

Clone the [hyle](https://github.com/Hyle-org/hyle) repo, checkout the version you need, and run:
```sh 
export RISC0_DEV_MODE=1
cargo run -- --pg
```

## Quickstart

### Build and register the contract

To build and register the smart contract on the local node, run:

```bash
cargo run -- register-contract
```

The expected output on the node is `📝 Registering contract counter`.


### Executing the Project Locally in Development Mode

During development, faster iteration upon code changes can be achieved by leveraging [dev-mode], we strongly suggest activating it during your early development phase.

```bash
RISC0_DEV_MODE=1 cargo run
```

### Execute the contract & send a tx on-chain

```sh
RISC0_DEV_MODE=1 cargo run -- increment
```

## CLI commands

```bash
cargo run -- register-image "hash_original_image" "image_signature" "owner_public_key""
```
Verifies the **image_signature**. If verified, adds the **hash_origianl_image** to the verified images with
**owner_pulbic_key** as its owner.


```bash
cargo run -- verify-original-image "hash_original_image"
```
verifies if the given **hash_origianl_image** is unedited.


```bash
cargo run -- add-publisher "hash_original_image" "digitalSig" "public_key_of_publisher"
```
Adds the publishing right of **hash_original_image** to the user with **public_key_of_publisher**.
Only works if the **hash_original_image** is digitally signed with **digitalSig** by the owner.


```bash
cargo run -- register-edit "hash_original_image" "hash_edited_image" "digitalSig"
```
Connects **hash_edited_image** to the original image **hash_original_image** if digitallySigned by a recognized publisher.


```bash
cargo run -- verify-edit-image "hash_edited_image" 
```
Checks if the hash_edited_image corresponds to some original image and responds the origianl image if found.

!Reusing the public key as the digital signature as a simplification!


## Directory Structure

It is possible to organize the files for these components in various ways.
However, in this starter template we use a standard directory structure for zkVM
applications, which we think is a good starting point for your applications.

```text
project_name
├── Cargo.toml
├── contract 
│   ├── Cargo.toml
│   └── src
│       └── lib.rs         <-- [Contract code goes here, common to host & guest]
├── host
│   ├── Cargo.toml
│   └── src
│       └── main.rs        <-- [Host code goes here]
└── methods
    ├── Cargo.toml
    ├── build.rs
    ├── guest
    │   ├── Cargo.toml
    │   └── src
    │       └── main.rs    <-- [Guest code goes here]
    └── src
        └── lib.rs
```

<!--[bonsai access]: https://bonsai.xyz/apply-->
[cargo-risczero]: https://docs.rs/cargo-risczero
[crates]: https://github.com/risc0/risc0/blob/main/README.md#rust-binaries
[dev-docs]: https://dev.risczero.com
[dev-mode]: https://dev.risczero.com/api/generating-proofs/dev-mode
[docs.rs]: https://docs.rs/releases/search?query=risc0
[examples]: https://github.com/risc0/risc0/tree/main/examples
[risc0-build]: https://docs.rs/risc0-build
[risc0-repo]: https://www.github.com/risc0/risc0
[risc0-zkvm]: https://docs.rs/risc0-zkvm
[rust-toolchain]: rust-toolchain.toml
[rustup]: https://rustup.rs
[zkvm-overview]: https://dev.risczero.com/zkvm
