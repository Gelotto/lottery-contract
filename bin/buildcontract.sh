#!/bin/bash

#Build Flag
PARAM=$1
####################################    Constants    ##################################################

#depends on mainnet or testnet
NODE="--node https://rpc-juno.itastakers.com:443"
CHAIN_ID=juno-1
DENOM="ujuno"

##########################################################################################

# NODE="--node https://rpc.juno.giansalex.dev:443"
# CHAIN_ID=uni-3
# DENOM="ujunox"

##########################################################################################
#not depends
NODECHAIN=" $NODE --chain-id $CHAIN_ID"
TXFLAG=" $NODECHAIN --gas-prices 0.003$DENOM --gas auto --gas-adjustment 1.3"
WALLET="--from gelotto"

RELEASE="release/"

WASMRAWFILE="cw_gelotto_ibc_lottery_smart_contract.wasm"
WASMFILE=$RELEASE$WASMRAWFILE

FILE_UPLOADHASH="uploadtx.txt"
FILE_CONTRACT_ADDR="contractaddr.txt"
FILE_CODE_ID="code.txt"

ADDR_GELOTTO="juno1uu20jhzj2s4rklhs4z29dwltfatt5zj8dvqgnq"
ADDR_ADMIN=$ADDR_GELOTTO

###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################
#Environment Functions
CreateEnv() {
    sudo apt-get update && sudo apt upgrade -y
    sudo apt-get install make build-essential gcc git jq chrony -y
    wget https://golang.org/dl/go1.17.3.linux-amd64.tar.gz
    sudo tar -C /usr/local -xzf go1.17.3.linux-amd64.tar.gz
    rm -rf go1.17.3.linux-amd64.tar.gz

    export GOROOT=/usr/local/go
    export GOPATH=$HOME/go
    export GO111MODULE=on
    export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin
    
    rustup default stable
    rustup target add wasm32-unknown-unknown

    git clone https://github.com/CosmosContracts/juno
    cd juno
    git fetch
    git checkout v6.0.0
    make install
    cd ../
    rm -rf juno
}

Upload() {
    echo "================================================="
    echo "Rust Optimize Build Start"
    mkdir release
    # RUSTFLAGS='-C link-arg=-s' cargo wasm
    # cp target/wasm32-unknown-unknown/$WASMFILE $WASMFILE

    docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.12.6
    cp artifacts/$WASMRAWFILE $WASMFILE
    
    

    echo "================================================="
    echo "Upload $WASMFILE"
    
    UPLOADTX=$(junod tx wasm store $WASMFILE $WALLET $TXFLAG --output json -y | jq -r '.txhash')
    echo "Upload txHash:"$UPLOADTX
    
    echo "================================================="
    echo "GetCode"
	CODE_ID=""
    while [[ $CODE_ID == "" ]]
    do 
        sleep 3
        CODE_ID=$(junod query tx $UPLOADTX $NODECHAIN --output json | jq -r '.logs[0].events[-1].attributes[0].value')
    done
    echo "Contract Code_id:"$CODE_ID

    #save to FILE_CODE_ID
    echo $CODE_ID > $FILE_CODE_ID
}

Instantiate() { 
    echo "================================================="
    echo "Instantiate Contract"
    #read from FILE_CODE_ID
    CODE_ID=$(cat $FILE_CODE_ID)
    echo $CODE_ID
    #read from FILE_CODE_ID
    
    TXHASH=$(junod tx wasm instantiate $CODE_ID '{"name":"DEF", "symbol":"DEF", "decimals":6, "deflation_rate":500, "new_account_deflation_rate":5,"new_account_deflation_time":86400}' --label "DeflationToken" --admin $ADDR_ADMIN $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    echo $TXHASH
    CONTRACT_ADDR=""
    while [[ $CONTRACT_ADDR == "" ]]
    do
        sleep 3
        CONTRACT_ADDR=$(junod query tx $TXHASH $NODECHAIN --output json | jq -r '.logs[0].events[0].attributes[0].value')
    done
    echo $CONTRACT_ADDR
    echo $CONTRACT_ADDR > $FILE_CONTRACT_ADDR
}


###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################


#################################################################################
PrintWalletBalance() {
    echo "native balance"
    echo "========================================="
    junod query bank balances $ADDR_ADMIN $NODECHAIN
    echo "========================================="
}

#################################### End of Function ###################################################
if [[ $PARAM == "" ]]; then
    Upload
    Instantiate
    PrintWalletBalance
else
    $PARAM
fi
