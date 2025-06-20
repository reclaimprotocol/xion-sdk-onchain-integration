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

        const owner = "0x612c00c6d44fa281beeea91805349519ef3c3e83"

        const claimInfo = {
            "provider": "http",
            "parameters": "{\"additionalClientOptions\":{},\"body\":\"\",\"geoLocation\":\"IN\",\"headers\":{\"Sec-Fetch-Mode\":\"same-origin\",\"Sec-Fetch-Site\":\"same-origin\",\"User-Agent\":\"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36\"},\"method\":\"GET\",\"paramValues\":{\"URL_PARAMS_1\":\"xWw45l6nX7DP2FKRyePXSw\",\"URL_PARAM_2_GRD\":\"variables=%7B%22screen_name%22%3A%22burnt9507278342%22%7D&features=%7B%22hidden_profile_subscriptions_enabled%22%3Atrue%2C%22profile_label_improvements_pcf_label_in_post_enabled%22%3Atrue%2C%22rweb_tipjar_consumption_enabled%22%3Atrue%2C%22verified_phone_label_enabled%22%3Afalse%2C%22subscriptions_verification_info_is_identity_verified_enabled%22%3Atrue%2C%22subscriptions_verification_info_verified_since_enabled%22%3Atrue%2C%22highlights_tweets_tab_ui_enabled%22%3Atrue%2C%22responsive_web_twitter_article_notes_tab_enabled%22%3Atrue%2C%22subscriptions_feature_can_gift_premium%22%3Atrue%2C%22creator_subscriptions_tweet_preview_api_enabled%22%3Atrue%2C%22responsive_web_graphql_skip_user_profile_image_extensions_enabled%22%3Afalse%2C%22responsive_web_graphql_timeline_navigation_enabled%22%3Atrue%7D&fieldToggles=%7B%22withAuxiliaryUserLabels%22%3Atrue%7D\",\"URL_PARAM_DOMAIN\":\"x\",\"created_at\":\"Wed Apr 23 16:06:50 +0000 2025\",\"followers_count\":\"0\",\"screen_name\":\"Burnt9507278342\"},\"responseMatches\":[{\"invert\":false,\"type\":\"contains\",\"value\":\"\\\"screen_name\\\":\\\"{{screen_name}}\\\"\"},{\"invert\":false,\"type\":\"contains\",\"value\":\"\\\"followers_count\\\":{{followers_count}}\"},{\"invert\":false,\"type\":\"contains\",\"value\":\"\\\"created_at\\\":\\\"{{created_at}}\\\"\"}],\"responseRedactions\":[{\"jsonPath\":\"$.data.user.result.core.screen_name\",\"regex\":\"\\\"screen_name\\\":\\\"(.*)\\\"\",\"xPath\":\"\"},{\"jsonPath\":\"$.data.user.result.legacy.followers_count\",\"regex\":\"\\\"followers_count\\\":(.*)\",\"xPath\":\"\"},{\"jsonPath\":\"$.data.user.result.core.created_at\",\"regex\":\"\\\"created_at\\\":\\\"(.*)\\\"\",\"xPath\":\"\"}],\"url\":\"https://{{URL_PARAM_DOMAIN}}.com/i/api/graphql/{{URL_PARAMS_1}}/UserByScreenName?{{URL_PARAM_2_GRD}}\"}",
            "context": "{\"extractedParameters\":{\"URL_PARAMS_1\":\"xWw45l6nX7DP2FKRyePXSw\",\"URL_PARAM_2_GRD\":\"variables=%7B%22screen_name%22%3A%22burnt9507278342%22%7D&features=%7B%22hidden_profile_subscriptions_enabled%22%3Atrue%2C%22profile_label_improvements_pcf_label_in_post_enabled%22%3Atrue%2C%22rweb_tipjar_consumption_enabled%22%3Atrue%2C%22verified_phone_label_enabled%22%3Afalse%2C%22subscriptions_verification_info_is_identity_verified_enabled%22%3Atrue%2C%22subscriptions_verification_info_verified_since_enabled%22%3Atrue%2C%22highlights_tweets_tab_ui_enabled%22%3Atrue%2C%22responsive_web_twitter_article_notes_tab_enabled%22%3Atrue%2C%22subscriptions_feature_can_gift_premium%22%3Atrue%2C%22creator_subscriptions_tweet_preview_api_enabled%22%3Atrue%2C%22responsive_web_graphql_skip_user_profile_image_extensions_enabled%22%3Afalse%2C%22responsive_web_graphql_timeline_navigation_enabled%22%3Atrue%7D&fieldToggles=%7B%22withAuxiliaryUserLabels%22%3Atrue%7D\",\"URL_PARAM_DOMAIN\":\"x\",\"created_at\":\"Wed Apr 23 16:06:50 +0000 2025\",\"followers_count\":\"0\",\"screen_name\":\"Burnt9507278342\"},\"providerHash\":\"0xd4fb71de874115b581e7c15fedd0f71b38fbfabf6894487d275fde2cca1d0ebb\"}",
        }

        const identifier = "0x5fba1c86439db035389d90f8025739c54849db4cfb7cf91aa3fb02abd9c1f83a"

        const signedClaim = {
            "claim": {
                "identifier": identifier,
                "owner": owner,
                "epoch": 1,
                "timestampS": 1748539856
            },
            "signatures": ["0x04fac06fb875a8a4896912461655f039b9b7726b1eacc1727f4b87c04b3971951387dc60b884e80e5c866722c1e34738a41c163f6c6bca2e33759a5ed34538201b"],
        }


        const proof = {
            claimInfo: claimInfo,
            signedClaim: signedClaim
        }

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