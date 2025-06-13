const { SigningCosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { GasPrice } = require("@cosmjs/stargate")
const { ReclaimClient } = require('@reclaimprotocol/zk-fetch')
const { transformForOnchain } = require('@reclaimprotocol/js-sdk')

const config = require('./config');

const APP_ID = "0x381994d6B9B08C3e7CfE3A4Cd544C85101b8f201"
const APP_SECRET = "0xfdc676e00ac9c648dfbcc271263c2dd95233a8abd391458c91ea88526a299223"

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

        // Initialize the ReclaimClient
        const reclaimClient = new ReclaimClient(
            APP_ID,
            APP_SECRET
        )

        // Example URL to fetch the data from
        const url =
            'https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd'

        // Generate the proof
        const proofData = await reclaimClient.zkFetch(
            url,
            { method: 'GET' },
            {
                responseMatches: [
                    {
                        type: 'regex',
                        value: '\\{"ethereum":\\{"usd":(?<price>[\\d\\.]+)\\}\\}'
                    }
                ]
            }
        )

        const proof = await transformForOnchain(proofData)

        const executeMsg = {
            "verify_proof": {
                proof: proof
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