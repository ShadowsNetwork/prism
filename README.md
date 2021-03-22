# Introduction
Shadows is a decentralized synthetic asset issuance protocol developed based on Substrate. These synthetic assets are guaranteed by the Shadows Network pass DOWS. As long as DOWS is locked in, synthetic assets can be issued.

# 1. Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Make sure you have `submodule.recurse` set to true to make life with submodule easier.

```bash
git config --global submodule.recurse true
```

Install required tools and install git hooks:

```bash
make init
```

Build all native code:

```bash
make build
```

# 2. Run

You can start a development chain with:

```bash
make run
```

# 3. Development

To type check:

```bash
make check
```

To purge old chain data:

```bash
make purge
```

To purge old chain data and run

```bash
make restart
```

# 4. Docker
An alternative to building locally is to use docker to run a pre-build binary. The only requirement is to have Docker installed.
```
# Pull the docker image
docker pull shadowsnetwork/shadows-parachain-testnet:latest

# Start a dev node
docker run --rm --network host --dev
```

# 5. Tests
Tests are run with the following command:
```
cargo test
```
