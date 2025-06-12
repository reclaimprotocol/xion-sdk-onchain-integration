# **Reclaim-XION Integration**

Reclaim Verifier on top of XION network.

## Deployments

| Chain Name | Deployed Address | Explorer Link |
|:-----------|:-----------------|:--------------|
| xion-testnet-2 | xion1zcphm9wllvmtlhclvwh8qrlcx46p5mwe4yfdve0c95xfh6arsejspq5nvd | https://explorer.burnt.com/xion-testnet-2/account/xion1zcphm9wllvmtlhclvwh8qrlcx46p5mwe4yfdve0c95xfh6arsejspq5nvd|

## **Prerequisites**

Before deploying the contract, ensure you have the following:

1. **XION Daemon (`xiond`)**  
   Follow the official guide to install `xiond`:  
   [Interact with XION Chain: Setup XION Daemon](https://docs.burnt.com/xion/developers/featured-guides/setup-local-environment/interact-with-xion-chain-setup-xion-daemon)

2. **Docker**  
   Install and run [Docker](https://www.docker.com/get-started), as it is required to compile the contract.

---

## **Deploy and Instantiate the Contract**

### **Step 1: Clone the Repository**
```sh
git clone https://github.com/reclaimprotocol/xion-sdk-onchain-integration.git
cd xion-sdk-onchain-integration
```

---

### **Step 2: Compile and Optimize the Wasm Bytecode**
Run the following command to compile and optimize the contract:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0
```

> **Note:**  
> This step uses **CosmWasm's Optimizing Compiler**, which reduces the contract's binary size, making it more efficient for deployment.  
> Learn more [here](https://github.com/CosmWasm/optimizer).

The optimized contract will be stored as:
```
/artifacts/reclaim_xion.wasm
```

---

### **Step 3: Upload the Bytecode to the Blockchain**
First, set your wallet address:
```sh
WALLET="your-wallet-address-here"
```

Now, upload the contract to the blockchain:
```sh
RES=$(xiond tx wasm store ./artifacts/reclaim_xion.wasm \
      --chain-id xion-testnet-2 \
      --gas-adjustment 1.3 \
      --gas-prices 0.001uxion \
      --gas auto \
      -y --output json \
      --node https://rpc.xion-testnet-2.burnt.com:443 \
      --from $WALLET)
```

After running the command, **extract the transaction hash**:
```sh
echo $RES
```

Example output:
```json
{
  "height": "0",
  "txhash": "B557242F3BBF2E68D228EBF6A792C3C617C8C8C984440405A578FBBB8A385035",
  ...
}
```

Copy the transaction hash for the next step.

---

### **Step 4: Retrieve the Code ID**
Set your transaction hash:
```sh
TXHASH="your-txhash-here"
```

Query the blockchain to get the **Code ID**:
```sh
CODE_ID=$(xiond query tx $TXHASH \
  --node https://rpc.xion-testnet-1.burnt.com:443 \
  --output json | jq -r '.events[-1].attributes[1].value')
```

Now, display the retrieved Code ID:
```sh
echo $CODE_ID
```

Example output:
```
1248
```

---

### **Step 5: Instantiate the Contract**
Set the contract's initialization message (replace with your own):
```sh
MSG='{"owner":"xion138lsa3mczwzl9j7flytg0f5r6pldaa8htddq0j"}'
```

Instantiate the contract with the **Code ID** from the previous step:
```sh
xiond tx wasm instantiate $CODE_ID "$MSG" \
  --from $WALLET \
  --label "reclaim_xion" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y --no-admin \
  --chain-id xion-testnet-1 \
  --node https://rpc.xion-testnet-1.burnt.com:443
```

Example output:
```
gas estimate: 217976
code: 0
txhash: 09D48FE11BE8D8BD4FCE11D236D80D180E7ED7707186B1659F5BADC4EC116F30
```

Copy the new transaction hash for the next step.

---

### **Step 6: Retrieve the Contract Address**
Set the new transaction hash:
```sh
TXHASH="your-txhash-here"
```

Query the blockchain to get the **contract address**:
```sh
CONTRACT=$(xiond query tx $TXHASH \
  --node https://rpc.xion-testnet-1.burnt.com:443 \
  --output json | jq -r '.events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
```

Display the contract address:
```sh
echo $CONTRACT
```

Example output:
```
xion1zcphm9wllvmtlhclvwh8qrlcx46p5mwe4yfdve0c95xfh6arsejspq5nvd
```

## **Interact with the Contract**

### **Step 1: Setup**

Change directory to the `/node` directory and install packages:

```sh
cd node

npm install
```

Then add your credentials to your `.env`:

```sh
XION_MNEMONIC="your wallet mnemonic phrase here"
XION_RPC_URL=https://rpc.xion-testnet-2.burnt.com
XION_CHAIN_ID=xion-testnet-2
CONTRACT_ADDRESS=xion1zcphm9wllvmtlhclvwh8qrlcx46p5mwe4yfdve0c95xfh6arsejspq5nvd
```

### **Step 2: Add an Epoch**

While in `/node`, run the command:

```sh
node add-epoch.js
```

This will add an Epoch, here is an example [tx](https://explorer.burnt.com/xion-testnet-2/tx/DB83FECEC90208D6438D09FFE8C3982B32D5193170F7055CE3A2AA460D7FB268). 

You can fetch an added Epoch with:

```sh
node get-epoch.js
```
### **Step 3: Verify a Proof**

While in the same directory, run the command:

```sh
node verify_proof.js 
```

This will add an Epoch, here is an example [tx](https://explorer.burnt.com/xion-testnet-2/tx/CD29667D64FEC7023F7F2713510FE3C2B41C47AA67CC4D5C96EE3092D139DEBD). 