# Substrate DEX Node
This project is intended for the Polkadot Blockchain Academy final assesment.
It is based on the substrate-node-template.

# Novelties
We all know the standard node-template README, so I'll just focus on whats new and different in this one.

- Bumped substrate branch from polkadot-v0.9.26 to polkadot-v0.9.27, unlike standard substrate-node-template (as of 2022-07-28)
- Relies on pallet-assets
- Block time of 500ms for that low latency trading goodness, may not be the best IRL though due to partitioning concerns due to global latencies