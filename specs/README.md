# Embedded Spec Files

This directory contains chain specs for well-known public networks.

## Context

The Ombre node is designed to support multiple networks including Ombre Alpha, Sombra
(Kusama) and Shadows (Polkadot). Some of these networks are already live and others are planned.

In order to support multiple networks with the same binary, Ombre relies on a chain specification
to know which network to sync. Rather than require node operators to obtain spec files separately,
it is convenient to "bake" specs for popular networks into the node.

## Which specs will come pre-baked?

- Ombre Alpha V4 - live
- Ombra - Potential future deployment to Rococo
- Sombra - Future Kusama Deployment
- Shadows - Future Polkadot deployment

## Relay chain specs

Because Ombre networks are parachains, each network instance requires both a parachain and a
relay chain spec. For popular relay chains like kusama and polkadot, we rely on the specs being
already included with Polkadot. For smaller relay chains, like the one that exists solely to support
ombre alpha, we also bake the relay spec into the ombre binary.
