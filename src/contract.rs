use {
    crate::state::{get_all_epochs, CONFIG, EPOCHS},
    cosmwasm_std::entry_point,
    cosmwasm_std::to_json_binary,
    cosmwasm_std::{
        Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult, Timestamp,
        Uint128,
    },
};

use crate::state::{Epoch, Witness};
use crate::{error::ContractError, msg::GetAllEpochResponse};
use crate::{
    msg::{ExecuteMsg, GetEpochResponse, InstantiateMsg, ProofMsg, QueryMsg},
    state::Config,
};
use sha2::{Digest, Sha256};

// version info for migration info
// const CONTRACT_NAME: &str = "crates.io:reclaim-cosmwasm";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let addr = deps.api.addr_validate(&msg.owner)?;
    let config = Config {
        owner: addr,
        current_epoch: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::VerifyProof(msg) => verify_proof(deps, msg, env),
        ExecuteMsg::AddEpoch {
            witness,
            minimum_witness,
        } => add_epoch(deps, env, witness, minimum_witness, info.sender.clone()),
    }
}

fn generate_random_seed(bytes: Vec<u8>, offset: usize) -> u32 {
    // Convert the hash result into a u32 using the offset
    let hash_slice = &bytes[offset..offset + 4];
    let mut seed = 0u32;
    for (i, &byte) in hash_slice.iter().enumerate() {
        seed |= u32::from(byte) << (i * 8);
    }

    seed
}

pub fn fetch_witness_for_claim(
    epoch: Epoch,
    identifier: String,
    timestamp: Timestamp,
) -> Vec<Witness> {
    let mut selected_witness = vec![];

    // Create a hash from identifier+epoch+minimum+timestamp
    let hash_str = format!(
        "{}\n{}\n{}\n{}",
        hex::encode(identifier),
        epoch.minimum_witness_for_claim_creation.to_string(),
        timestamp.nanos().to_string(),
        epoch.id.to_string()
    );
    let result = hash_str.as_bytes().to_vec();
    let mut hasher = Sha256::new();
    hasher.update(result);
    let hash_result = hasher.finalize().to_vec();
    let witenesses_left_list = epoch.witness;
    let mut byte_offset = 0;
    let witness_left = witenesses_left_list.len();
    for _i in 0..epoch.minimum_witness_for_claim_creation.into() {
        let random_seed = generate_random_seed(hash_result.clone(), byte_offset) as usize;
        let witness_index = random_seed % witness_left;
        let witness = witenesses_left_list.get(witness_index);
        match witness {
            Some(data) => selected_witness.push(data.clone()),
            None => {}
        }
        byte_offset = (byte_offset + 4) % hash_result.len();
    }

    selected_witness
}

pub fn verify_proof(deps: DepsMut, msg: ProofMsg, env: Env) -> Result<Response, ContractError> {
    // Find the epoch from database
    let epoch = EPOCHS.load(deps.storage, msg.proof.signedClaim.claim.epoch.into())?;
    let mut resp = Response::new();

    // Hash the claims, and verify with identifier hash
    let hashed = msg.proof.claimInfo.hash();
    if msg.proof.signedClaim.claim.identifier != hashed {
        return Err(ContractError::HashMismatchErr {});
    }

    // Fetch witness for claim
    let expected_witness = fetch_witness_for_claim(
        epoch,
        msg.proof.signedClaim.claim.identifier.clone(),
        env.block.time,
    );

    let expected_witness_addresses = Witness::get_addresses(expected_witness);

    // recover witness address from SignedClaims Object
    let signed_witness = msg
        .proof
        .signedClaim
        .recover_signers_of_signed_claim(deps)?;

    // make sure the minimum requirement for witness is satisfied
    if expected_witness_addresses.len() != signed_witness.len() {
        return Err(ContractError::WitnessMismatchErr {});
    }

    // Ensure for every signature in the sign, a expected witness exists from the database
    for signed in signed_witness {
        let signed_event = Event::new("signer").add_attribute("sig", signed.clone());
        resp = resp.add_event(signed_event);
        if !expected_witness_addresses.contains(&signed) {
            return Err(ContractError::SignatureErr {});
        }
    }
    Ok(resp)
}

// @dev - add epoch
pub fn add_epoch(
    deps: DepsMut,
    env: Env,
    witness: Vec<Witness>,
    minimum_witness: Uint128,
    sender: Addr,
) -> Result<Response, ContractError> {
    // load configs
    let mut config = CONFIG.load(deps.storage)?;

    // Check if sender is owner
    if config.owner != sender {
        return Err(ContractError::Unauthorized {});
    }

    //Increment Epoch number
    let new_epoch = config.current_epoch + Uint128::one();
    // Create the new epoch
    let epoch = Epoch {
        id: new_epoch,
        witness,
        timestamp_start: env.block.time.nanos(),
        timestamp_end: env.block.time.plus_days(1).nanos(),
        minimum_witness_for_claim_creation: minimum_witness,
    };

    // Upsert the new epoch into memory
    EPOCHS.update(
        deps.storage,
        new_epoch.into(),
        // we check if epoch with same id already exists for safety
        |existsting| match existsting {
            None => Ok(epoch),
            Some(..) => Err(ContractError::AlreadyExists {}),
        },
    )?;

    // Save the new epoch
    config.current_epoch = new_epoch;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEpoch { id } => to_json_binary(&query_epoch_id(deps, id)?),
        QueryMsg::GetAllEpoch {} => to_json_binary(&query_all_epoch_ids(deps)?),
    }
}

fn query_all_epoch_ids(deps: Deps) -> StdResult<GetAllEpochResponse> {
    Ok(GetAllEpochResponse {
        ids: get_all_epochs(deps.storage)?,
    })
}

fn query_epoch_id(deps: Deps, id: u128) -> StdResult<GetEpochResponse> {
    let data = EPOCHS.load(deps.storage, id)?;
    Ok(GetEpochResponse { epoch: data })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::{ClaimInfo, CompleteClaimData, Proof, SignedClaim};
    use crate::state::{CONFIG, EPOCHS};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, StdError, Uint128};

    const OWNER: &str = "owner0000";
    const USER: &str = "user0000";
    const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
    const RECLAIM_ADDRESS: &str = "0x244897572368eadf65bfbc5aec98d8e5443a9072";

    // Helper to instantiate contract with default owner
    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            owner: OWNER.to_string(),
        };
        let info = mock_info(OWNER, &[]);
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    fn create_test_epoch() -> Epoch {
        Epoch {
            id: Uint128::from(1u128),
            witness: vec![Witness {
                address: RECLAIM_ADDRESS.to_string(), // Signer's address
                host: "https://valid-witness.com".to_string(),
            }],
            timestamp_start: 0,
            timestamp_end: 0,
            minimum_witness_for_claim_creation: Uint128::from(1u128),
        }
    }

    fn create_proof_msg() -> ProofMsg {
        ProofMsg {proof: Proof{
            claimInfo: ClaimInfo {
                provider: "http".to_string(),
                parameters: r#"{"additionalClientOptions":{},"body":"","geoLocation":"IN","headers":{"Sec-Fetch-Mode":"same-origin","Sec-Fetch-Site":"same-origin","User-Agent":"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36"},"method":"GET","paramValues":{"URL_PARAMS_1":"xWw45l6nX7DP2FKRyePXSw","URL_PARAM_2_GRD":"variables=%7B%22screen_name%22%3A%22burnt9507278342%22%7D&features=%7B%22hidden_profile_subscriptions_enabled%22%3Atrue%2C%22profile_label_improvements_pcf_label_in_post_enabled%22%3Atrue%2C%22rweb_tipjar_consumption_enabled%22%3Atrue%2C%22verified_phone_label_enabled%22%3Afalse%2C%22subscriptions_verification_info_is_identity_verified_enabled%22%3Atrue%2C%22subscriptions_verification_info_verified_since_enabled%22%3Atrue%2C%22highlights_tweets_tab_ui_enabled%22%3Atrue%2C%22responsive_web_twitter_article_notes_tab_enabled%22%3Atrue%2C%22subscriptions_feature_can_gift_premium%22%3Atrue%2C%22creator_subscriptions_tweet_preview_api_enabled%22%3Atrue%2C%22responsive_web_graphql_skip_user_profile_image_extensions_enabled%22%3Afalse%2C%22responsive_web_graphql_timeline_navigation_enabled%22%3Atrue%7D&fieldToggles=%7B%22withAuxiliaryUserLabels%22%3Atrue%7D","URL_PARAM_DOMAIN":"x","created_at":"Wed Apr 23 16:06:50 +0000 2025","followers_count":"0","screen_name":"Burnt9507278342"},"responseMatches":[{"invert":false,"type":"contains","value":"\"screen_name\":\"{{screen_name}}\""},{"invert":false,"type":"contains","value":"\"followers_count\":{{followers_count}}"},{"invert":false,"type":"contains","value":"\"created_at\":\"{{created_at}}\""}],"responseRedactions":[{"jsonPath":"$.data.user.result.core.screen_name","regex":"\"screen_name\":\"(.*)\"","xPath":""},{"jsonPath":"$.data.user.result.legacy.followers_count","regex":"\"followers_count\":(.*)","xPath":""},{"jsonPath":"$.data.user.result.core.created_at","regex":"\"created_at\":\"(.*)\"","xPath":""}],"url":"https://{{URL_PARAM_DOMAIN}}.com/i/api/graphql/{{URL_PARAMS_1}}/UserByScreenName?{{URL_PARAM_2_GRD}}"}"#.to_string(),
                context: r#"{"extractedParameters":{"URL_PARAMS_1":"xWw45l6nX7DP2FKRyePXSw","URL_PARAM_2_GRD":"variables=%7B%22screen_name%22%3A%22burnt9507278342%22%7D&features=%7B%22hidden_profile_subscriptions_enabled%22%3Atrue%2C%22profile_label_improvements_pcf_label_in_post_enabled%22%3Atrue%2C%22rweb_tipjar_consumption_enabled%22%3Atrue%2C%22verified_phone_label_enabled%22%3Afalse%2C%22subscriptions_verification_info_is_identity_verified_enabled%22%3Atrue%2C%22subscriptions_verification_info_verified_since_enabled%22%3Atrue%2C%22highlights_tweets_tab_ui_enabled%22%3Atrue%2C%22responsive_web_twitter_article_notes_tab_enabled%22%3Atrue%2C%22subscriptions_feature_can_gift_premium%22%3Atrue%2C%22creator_subscriptions_tweet_preview_api_enabled%22%3Atrue%2C%22responsive_web_graphql_skip_user_profile_image_extensions_enabled%22%3Afalse%2C%22responsive_web_graphql_timeline_navigation_enabled%22%3Atrue%7D&fieldToggles=%7B%22withAuxiliaryUserLabels%22%3Atrue%7D","URL_PARAM_DOMAIN":"x","created_at":"Wed Apr 23 16:06:50 +0000 2025","followers_count":"0","screen_name":"Burnt9507278342"},"providerHash":"0xd4fb71de874115b581e7c15fedd0f71b38fbfabf6894487d275fde2cca1d0ebb"}"#.to_string(),
                 },
            signedClaim: SignedClaim {
                claim: CompleteClaimData {
                    identifier: "0x5fba1c86439db035389d90f8025739c54849db4cfb7cf91aa3fb02abd9c1f83a".to_string(),
                    owner: "0x612c00c6d44fa281beeea91805349519ef3c3e83".to_string(),
                    epoch: 1_u64,
                    timestampS: 1748539856,
                },
                signatures: vec![
                    "0x04fac06fb875a8a4896912461655f039b9b7726b1eacc1727f4b87c04b3971951387dc60b884e80e5c866722c1e34738a41c163f6c6bca2e33759a5ed34538201b".to_string()
                ],
            },
        }
    }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: OWNER.to_string(),
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(0, res.messages.len());

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.owner, Addr::unchecked(OWNER));
        assert_eq!(config.current_epoch, Uint128::zero());
    }

    #[test]
    fn initialization_with_invalid_owner() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: "x".repeat(1000).to_string(), // Invalid address format
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);

        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            ContractError::Std(StdError::generic_err("Invalid input: human address too long for this mock implementation (must be <= 90)."))
        );
    }

    #[test]
    fn add_epoch_successfully() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        let witnesses = vec![
            Witness {
                address: ZERO_ADDRESS.to_string(),
                host: "https://w1.com".to_string(),
            },
            Witness {
                address: RECLAIM_ADDRESS.to_string(),
                host: "https://w2.com".to_string(),
            },
        ];

        let info = mock_info(OWNER, &[]);
        let msg = ExecuteMsg::AddEpoch {
            witness: witnesses.clone(),
            minimum_witness: Uint128::from(2u128),
        };

        // First epoch
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Verify state changes
        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.current_epoch, Uint128::one());

        let epoch = EPOCHS.load(&deps.storage, 1).unwrap();
        assert_eq!(epoch.id, Uint128::one());
        assert_eq!(epoch.witness, witnesses);
        assert_eq!(
            epoch.minimum_witness_for_claim_creation,
            Uint128::from(2u128)
        );
        assert_eq!(epoch.timestamp_start, env.block.time.nanos());
        assert_eq!(epoch.timestamp_end, env.block.time.plus_days(1).nanos());
    }

    #[test]
    fn add_epoch_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let info = mock_info(USER, &[]); // Non-owner
        let msg = ExecuteMsg::AddEpoch {
            witness: vec![],
            minimum_witness: Uint128::from(1u128),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res, Err(ContractError::Unauthorized {}));
    }

    #[test]
    fn epoch_id_increments_correctly() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        let info = mock_info(OWNER, &[]);
        let msg = ExecuteMsg::AddEpoch {
            witness: vec![Witness {
                address: ZERO_ADDRESS.to_string(),
                host: "https://w.com".to_string(),
            }],
            minimum_witness: Uint128::one(),
        };

        // Add three epochs
        execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        execute(deps.as_mut(), env, info, msg).unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.current_epoch, Uint128::from(3u128));

        // Verify all epochs exist
        assert!(EPOCHS.has(&deps.storage, 1));
        assert!(EPOCHS.has(&deps.storage, 2));
        assert!(EPOCHS.has(&deps.storage, 3));
    }

    #[test]
    fn prevent_epoch_id_collision() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // Manually create conflicting epoch
        EPOCHS
            .save(
                &mut deps.storage,
                1,
                &Epoch {
                    id: Uint128::one(),
                    witness: vec![],
                    timestamp_start: 0,
                    timestamp_end: 0,
                    minimum_witness_for_claim_creation: Uint128::zero(),
                },
            )
            .unwrap();

        let info = mock_info(OWNER, &[]);
        let msg = ExecuteMsg::AddEpoch {
            witness: vec![],
            minimum_witness: Uint128::one(),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(res, Err(ContractError::AlreadyExists {}));
    }

    #[test]
    fn add_epoch_with_zero_witnesses() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let info = mock_info(OWNER, &[]);
        let msg = ExecuteMsg::AddEpoch {
            witness: vec![],
            minimum_witness: Uint128::zero(),
        };

        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let epoch = EPOCHS.load(&deps.storage, 1).unwrap();
        assert!(epoch.witness.is_empty());
        assert_eq!(epoch.minimum_witness_for_claim_creation, Uint128::zero());
    }

    #[test]
    fn query_epoch_after_creation() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let witness = vec![Witness {
            address: ZERO_ADDRESS.to_string(),
            host: "https://query.com".to_string(),
        }];

        let info = mock_info(OWNER, &[]);
        let msg = ExecuteMsg::AddEpoch {
            witness: witness.clone(),
            minimum_witness: Uint128::from(5u128),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Query the epoch
        let res = query_epoch_id(deps.as_ref(), 1).unwrap();
        assert_eq!(res.epoch.id, Uint128::one());
        assert_eq!(res.epoch.witness, witness);
    }

    #[test]
    fn verify_proof_successfully() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = "owner0000";
        let msg = InstantiateMsg {
            owner: owner.to_string(),
        };
        let info = mock_info(owner, &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        EPOCHS
            .save(deps.as_mut().storage, 1, &create_test_epoch())
            .unwrap();

        let proof = create_proof_msg();

        // Verify the proof
        let res = verify_proof(deps.as_mut(), proof, env.clone());

        // Should succeed
        assert!(res.is_ok());

        // Check events contain the expected signer
        let response = res.unwrap();
        let signer_event = response.events.iter().find(|e| e.ty == "signer");
        assert!(signer_event.is_some());
        assert_eq!(signer_event.unwrap().attributes[0].value, RECLAIM_ADDRESS);
    }

    #[test]
    fn verify_proof_with_hash_mismatch() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = "owner0000";
        let msg = InstantiateMsg {
            owner: owner.to_string(),
        };
        let info = mock_info(owner, &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        EPOCHS
            .save(deps.as_mut().storage, 1, &create_test_epoch())
            .unwrap();

        // Create modified proof with invalid identifier
        let mut proof = create_proof_msg();
        proof.proof.signedClaim.claim.identifier = "invalid_hash".to_string();

        // Verify should fail
        let res = verify_proof(deps.as_mut(), proof, env);
        assert_eq!(res, Err(ContractError::HashMismatchErr {}));
    }

    #[test]
    fn verify_proof_with_signature_error() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = "owner0000";
        let msg = InstantiateMsg {
            owner: owner.to_string(),
        };
        let info = mock_info(owner, &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        EPOCHS
            .save(deps.as_mut().storage, 1, &create_test_epoch())
            .unwrap();

        let mut proof = create_proof_msg();

        // Just change the recovery id from "1b" to "1c"
        proof.proof.signedClaim.signatures[0] = "0x04fac06fb875a8a4896912461655f039b9b7726b1eacc1727f4b87c04b3971951387dc60b884e80e5c866722c1e34738a41c163f6c6bca2e33759a5ed34538201c".to_string();

        // Verify should fail
        let res = verify_proof(deps.as_mut(), proof, env);
        assert_eq!(res, Err(ContractError::SignatureErr {}));
    }
}
