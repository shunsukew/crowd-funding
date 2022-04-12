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
