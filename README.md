
# Shadows Network

Run an Ethereum compatible parachain based on Substrate.

*See [www.substrate.io](https://www.substrate.io/) for substrate information.*

*See [shadows.link](https://shadows.link/) for the shadows blockchain description.*

# Local Development

Follow these steps to prepare a local Substrate development environment :hammer_and_wrench:

# Install (linux)

## Get the code

Get the master branch of shadows：

```bash
git clone https://github.com/ShadowsNetwork/shadows.git
cd shadows
```

## Simple Setup

Install all the required dependencies with a single command (be patient, this can take up to 30
minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
source $HOME/.cargo/env
```

Run the initialization script, which checks the correct rust nightly version and adds the `wasm32-unknown-unknown` target to that specific version:

```bash
./scripts/init.sh
```

### Build  the Shadows Node

Once the development environment is set up, build the shadows-node. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and [native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release --features "aura"
```

## Run

### Single Node Development Chain

Purge any existing dev chain state:

```bash
./target/release/shadows-node purge-chain --dev
```

Start a dev chain:

```bash
./target/release/shadows-node  --dev --ws-external --rpc-external --rpc-cors=all
```

Or, start a dev chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/shadows-node -lruntime=debug --dev
```

 


## deployment contract
Local TestNet:  127.0.0.1:9933 ,

ChainId: 888 ,

Account:

- Address: 0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4
- PriKey: 0x665c5c10437cc1220b805b3b6d015c82f476e1d8144f08ba85840eddf4b903a5
- contractAddress: 0x22b7265E52943D5A2F610bCf075F6AC307BcC706

if you want to deployment contract on the testnet,this will help you.


## TEST

Start a test chain with "manual-seal":
```bash
cargo build --release --features "manual-seal"
```

if you want to test, it is a good idea to use  https://hardhat.org/tutorial/

After setting up the hardhat test environment

```bash
npx hardhat test
```

