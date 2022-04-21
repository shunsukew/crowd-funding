# CosmWasm Starter Pack

This is a template to build smart contracts in Rust to run inside a
[Cosmos SDK](https://github.com/cosmos/cosmos-sdk) module on all chains that enable it.
To understand the framework better, please read the overview in the
[cosmwasm repo](https://github.com/CosmWasm/cosmwasm/blob/master/README.md),
and dig into the [cosmwasm docs](https://www.cosmwasm.com).
This assumes you understand the theory and just want to get coding.

===

Using Cliff Network
https://block-explorer.cliffnet.cosmwasm.com

===

CW20

CW20_CODE_ID=1037
contract address: wasm18w478cawahsx2ju5jq6xfjsqk4rg0a8eq303qs30aywlykexsjrqc56g3u

```
INIT=$(jq -n --arg wallet $(wasmd keys show -a wallet) --arg wallet2 $(wasmd keys show -a wallet2) '{"name":"test","symbol":"TST","decimals":8,"initial_balances":[{"address":$wallet,"amount":"100000000"},{"address":$wallet2,"amount":"100000000"}]}')

wasmd tx wasm instantiate $CODE_ID $INIT --label 'test' $TXFLAG -y --admin wasm1285yz3efp8t0aaqqwd5qyedv6g4val0f2e0z3z --from wallet

CW20_CONTRACT=$(wasmd query wasm list-contract-by-code $CW20_CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CW20_CONTRACT

QUERY=$(jq -n --arg wallet $(wasmd keys show -a wallet) '{"balance":{"address":$wallet}}')
wasmd query wasm contract-state smart $CW20_CONTRACT $QUERY $NODE

EXECUTE='{"transfer":{"recipient":"wasmADDRESSOFRECIPIENT","amount":"10"}}'
wasmd tx wasm execute $CW20_CONTRACT $EXECUTE --from wallet $TXFLAG -y

```

===

Simple Crowd Funding Contract

How to interact with contracts
https://docs.cosmwasm.com/docs/1.0/getting-started/interact-with-contract


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
export CROWDFUNDING_CODE_ID=1038

# Init Message
INIT='{"token":{"CW20":{"addr":"wasm18w478cawahsx2ju5jq6xfjsqk4rg0a8eq303qs30aywlykexsjrqc56g3u"}},"target_amount":"100","title":"Test Project CW20 Funding","description":"This is a test with cw20","end_time":1649741400}'

# Initialize, wallet address as an contract admin
wasmd tx wasm instantiate $CROWDFUNDING_CODE_ID "$INIT" \
    --from wallet --label "awesome crowd funding" $TXFLAG -y --admin wasm1285yz3efp8t0aaqqwd5qyedv6g4val0f2e0z3z
```

**Interact with Contract**
```
# Check Contract Address
CROWDFUNDING_CONTRACT=$(wasmd query wasm list-contract-by-code $CROWDFUNDING_CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CROWDFUNDING_CONTRACT

# Check Contract Info
wasmd query wasm contract $CROWDFUNDING_CONTRACT $NODE
wasmd query bank balances $CROWDFUNDING_CONTRACT $NODE

# you can dump entire contract state
wasmd query wasm contract-state all $CROWDFUNDING_CONTRACT $NODE

# Read
## Get Project Info
wasmd query wasm contract-state smart $CROWDFUNDING_CONTRACT '{"get_project_info":{}}' $NODE 

## Get Current Contribution
wasmd query wasm contract-state smart $CROWDFUNDING_CONTRACT '{"get_contribution":{"address":"wasm1vv8h0exmzvxhg4d0gvrctwg2ah9e7g38nw4ru6"}}' $NODE

# Write
## Contribute (Native Token case)
CONTRIBUTE='{"contribute":{}}'
wasmd tx wasm execute $CROWDFUNDING_CONTRACT "$CONTRIBUTE" \
    --amount 101upebble \
    --from wallet2 $TXFLAG -y

## CW20 case
echo $CW20_CONTRACT
echo $CROWDFUNDING_CONTRACT
EXECUTE='{"send":{"contract":"CROWDFUNDING_CONTRACT_ADDRESS","amount":"10000", "msg":""}}'
wasmd tx wasm execute $CW20_CONTRACT $EXECUTE --from wallet $TXFLAG -y

WITHDRAW='{"withdraw":{}}'
wasmd tx wasm execute $CROWDFUNDING_CONTRACT "$WITHDRAW" \
    --from wallet $TXFLAG -y

REFUND='{"refund":{}}'
wasmd tx wasm execute $CROWDFUNDING_CONTRACT "$REFUND" \
    --from wallet2 $TXFLAG -y
```

# Demo Procedure
Notion: https://www.notion.so/staketechnologies/CosmWasm-f9836b9d173547d78e4059a733f55ed8
Explorer: https://block-explorer.cliffnet.cosmwasm.com

## Templating
```
cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --name my-first-contract
cd my-first-contract
```

## Setup Env variables
```
RPC=https://rpc.cliffnet.cosmwasm.com:443

export NODE=(--node $RPC)
export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 0.025upebble --gas auto --gas-adjustment 1.3)
```

## Check Keys, Balances
```
wasmd keys list
wasmd query bank balances $(wasmd keys show -a wallet) $NODE
wasmd query bank balances $(wasmd keys show -a wallet2) $NODE
```

## Compile, Test
```
RUST_BACKTRACE=1 cargo unit-test
RUSTFLAGS='-C link-arg=-s' cargo wasm
```

## Upload Contract
```
RES=$(wasmd tx wasm store target/wasm32-unknown-unknown/release/crowd_funding.wasm --from wallet $TXFLAG -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json
```

## Instantiate
```
INIT='{"token":{"Native":{"denom":"upebble"}},"target_amount":"100","title":"Test Project Native Token Funding","description":"This is a test with native token","end_time":1649899800}'

wasmd tx wasm instantiate $CODE_ID "$INIT" \
    --from wallet --label "awesome crowd funding" $TXFLAG -y --admin $(wasmd keys show -a wallet)

CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CONTRACT

wasmd query wasm contract $CONTRACT $NODE
wasmd query bank balances $CONTRACT $NODE
```

## Query
```
wasmd query wasm contract-state smart $CONTRACT '{"get_project_info":{}}' $NODE 

wasmd query wasm contract-state smart $CONTRACT '{"get_contribution":{"address":"wasm1vv8h0exmzvxhg4d0gvrctwg2ah9e7g38nw4ru6"}}' $NODE
```

## Excute
```
# check balance of contributor
wasmd query bank balances $(wasmd keys show -a wallet2) $NODE

CONTRIBUTE='{"contribute":{}}'
wasmd tx wasm execute $CONTRACT "$CONTRIBUTE" \
    --amount 1000000upebble \
    --from wallet2 $TXFLAG -y

# Check balance of project owner
wasmd query bank balances $(wasmd keys show -a wallet) $NODE

WITHDRAW='{"withdraw":{}}'
wasmd tx wasm execute $CONTRACT "$WITHDRAW" \
    --from wallet $TXFLAG -y

# Check balance of project owner again
wasmd query bank balances $(wasmd keys show -a wallet) $NODE
```


# Terrain
https://docs.terra.money/docs/develop/dapp/quick-start/using-terrain-localterra.html
