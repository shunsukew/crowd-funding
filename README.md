# CosmWasm Starter Pack

This is a template to build smart contracts in Rust to run inside a
[Cosmos SDK](https://github.com/cosmos/cosmos-sdk) module on all chains that enable it.
To understand the framework better, please read the overview in the
[cosmwasm repo](https://github.com/CosmWasm/cosmwasm/blob/master/README.md),
and dig into the [cosmwasm docs](https://www.cosmwasm.com).
This assumes you understand the theory and just want to get coding.

===

Simple Crowd Funding Contract

How to interact with contracts
https://docs.cosmwasm.com/docs/1.0/getting-started/interact-with-contract

CODE_ID: 1029

```
RPC=https://rpc.cliffnet.cosmwasm.com:443

# bash
export NODE="--node $RPC"
export TXFLAG="${NODE} --chain-id ${CHAIN_ID} --gas-prices 0.025upebble --gas auto --gas-adjustment 1.3"

# zsh
export NODE=(--node $RPC)
export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 0.025upebble --gas auto --gas-adjustment 1.3)
```

**Initialize**
```
# Init Message
INIT='{"target_amount":{"amount":"100","denom":"upebble"},"title":"Test Project","description":"This is a test","end_time":1649667600}'

# Initialize, wallet address as an contract admin
wasmd tx wasm instantiate $CODE_ID "$INIT" \
    --from wallet --label "awesome crowd funding" $TXFLAG -y --admin wasm1285yz3efp8t0aaqqwd5qyedv6g4val0f2e0z3z
```

Cliff Network
Init Transaction: https://block-explorer.cliffnet.cosmwasm.com/transactions/995300FC809A98EF4A872395EBD3E8EEFB5F6A83349E63DB2907CC46E48E2C69

**Interact with Contract**
```
# Check Contract Address
CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CONTRACT

# Check Contract Info
wasmd query wasm contract $CONTRACT $NODE
wasmd query bank balances $CONTRACT $NODE

# you can dump entire contract state
wasmd query wasm contract-state all $CONTRACT $NODE


# Read
## Get Project Info
wasmd query wasm contract-state smart $CONTRACT '{"get_project_info":{}}' $NODE 

## Get Current Contribution
wasmd query wasm contract-state smart $CONTRACT '{"get_contribution":{"address":"wasm1vv8h0exmzvxhg4d0gvrctwg2ah9e7g38nw4ru6"}}' $NODE

# Write
## Contribute
CONTRIBUTE='{"contribute":{}}'
wasmd tx wasm execute $CONTRACT "$CONTRIBUTE" \
    --amount 101upebble \
    --from wallet2 $TXFLAG -y

WITHDRAW='{"withdraw":{}}'
wasmd tx wasm execute $CONTRACT "$WITHDRAW" \
    --from wallet $TXFLAG -y

REFUND='{"refund":{}}'
wasmd tx wasm execute $CONTRACT "$REFUND" \
    --from wallet2 $TXFLAG -y
```
