const { CosmWasmClient } = require("@cosmjs/cosmwasm-stargate");

const config = require('./config');

async function run() {
    try {

        const queryMsg = { "get_epoch": { "id": "1" } };

        const client = await CosmWasmClient.connect(config.XION_RPC_URL);
        const result = await client.queryContractSmart(config.CONTRACT_ADDRESS, queryMsg);
        
        console.log(result);

    } catch (error) {
        console.error("Error executing:", error);
        console.error(error.stack);
    }
}

run()