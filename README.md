# Substrate DEX Node
This project is intended for the Polkadot Blockchain Academy final assesment.
It is based on the substrate-node-template.

## Novelties
We all know the standard node-template README, so I'll just focus on whats new and different in this one.

- Bumped substrate branch from polkadot-v0.9.26 to polkadot-v0.9.27, unlike standard substrate-node-template (as of 2022-07-28)
- Relies on pallet-assets
- Block time of 500ms for that low latency trading goodness, may not be the best IRL though due to partitioning concerns due to global latencies
- Custom RPC endpoint integrated for querying the current price of a market
- Custom runtime-api which is obviously required by RPC
- Offchain worker which rewards liquidity providers every 10 blocks, from the fees collected
- 19 tests covering all Dispatchables and both the failure and successcases, also covering all storage changes

## Overview
The "pallet-dex" modules acts as an automated market maker 
where the product of the quantity of two assets remains constant as such: x * y = k.

To clarify the nomencalture used throughout the project it is important to know these:
- BASE and QUOTE asset are the two currencies in a market, for example BTCUSD, 
  in this case BTC is the BASE asset as its quoted in the QUOTE asset USD.
- Taker: Is the one taking liquidity and using a market order, aka user
- Liquidity Provider: Is the market maker, but in a more passive sense in AMMs

The functionality exposed as Dispatchables is as such:

- create_market_pool: Allows the user to create a liquidity pool for two assets with some initial two sided liquidity balances
- deposit_liquidity: Allows the user to add liquidity to a pool to earn part of the collected taker fees
- withdraw_liquidity: Allows the user to remove his liquidity from a pool again
- buy: Allows the user to exchange the QUOTE asset for the BASE asset 
  at an automatically determined exchange rate based on the balances in the pool
- sell: Allows the user to exchange the BASE asset for the QUOTE asset

Liquidity providers get rewarded by receiving a share of the collected taker fees.
This happens automatically every 10 block, triggered by the offchain_worker.

The RPC method that is exposed:
- current_price: Returns the current price of the market, assuming no slippage due to an order fill

## Cool to have in the future:
Given more time, there are a couple of cool things I'd would have liked to include such as:
- An external set of trading agents that interact and trade with the chain
 to simulate the correctness and behaviour of differing agents in an AMM setting.
