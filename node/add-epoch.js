const { SigningCosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { GasPrice } = require("@cosmjs/stargate")

const config = require('./config');

async function run() {
    try {

        const wallet = await DirectSecp256k1HdWallet.fromMnemonic(config.MNEMONIC, {
            prefix: "xion",
        });

        const [firstAccount] = await wallet.getAccounts();
        const senderAddress = firstAccount.address;

        const gasPrice = GasPrice.fromString("0.0025uxion");

        const client = await SigningCosmWasmClient.connectWithSigner(config.XION_RPC_URL, wallet,
            {
                gasPrice: gasPrice,
            }
        )

        const fee = "auto";

        const executeMsg = {
            "add_epoch": {
                    witness: [{ address: "0x244897572368eadf65bfbc5aec98d8e5443a9072", host: "https://reclaim-node.questbook.app" }],
                    minimum_witness: "1",
                }
        }

        const result = await client.execute(senderAddress, config.CONTRACT_ADDRESS, executeMsg, fee)

        console.log(result)

    } catch (error) {
        console.error("Error executing:", error);
        console.error(error.stack);
    }
}

run()