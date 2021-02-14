
## Install (linux)

### Get the code

Get the tutorial specific tag of the ShadowsNetwork/Shadows repo:

```bash
git clone -b tutorial-v3 https://github.com/ShadowsNetwork/shadows
cd shadows
```

### Setting up enviroment

Install Substrate pre-requisites (including Rust):

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

Run the initialization script, which checks the correct rust nightly version and adds the WASM to
that specific version:

```bash
./scripts/init.sh
```

## Build Standalone

Build the corresponding binary file:

```bash
cd node/standalone
cargo build --release
```

## Build Parachain

Build the corresponding binary file:

```bash
cargo build --release
```

The first build takes a long time, as it compiles all the necessary libraries.

### Troubleshooting

If a _cargo not found_ error appears in the terminal, manually add Rust to your system path (or
restart your system):

```bash
source $HOME/.cargo/env
```

## Run

### Standalone Node in dev mode

```bash
./node/standalone/target/release/ombre-standalone --dev
```

## Docker image

### Standlone node

An alternative to the steps higlighted before is to use docker to run a pre-build binary. Doing so, you prevent having to install Substrate and all the dependencies, and you can skip the building the node process as well. The only requirement is to have Docker installed, and then you can execute the following command to download the corresponding image:

```bash
docker pull shadowsnetwork/ombre:tutorial-v3
```

Once the Docker image is downloaded, you can run it with the following line:

```bash
docker run --rm --name shadows_standalone --network host shadowsnetwork/ombre:tutorial-v3 /ombre/ombre-standalone --dev
```
