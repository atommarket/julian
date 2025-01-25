//cargo tarpaulin --ignore-tests = 79.23% coverage, 290/366 lines covered
//2 tests fail due to the new way that cosmwasm deals with bech32 addresses (cosmwasmaddr1...). Uncomment test address and comment out legit addresses in contract.rs for testing.
use crate::contract::{execute, instantiate, migrate, query};
use crate::msg::{
    AllListingsResponse, ArbitrationListingsResponse, ExecuteMsg, InstantiateMsg,
    ListingCountResponse, ListingResponse, MigrateMsg, QueryMsg, SearchListingsResponse,
    ProfileResponse,
};
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{attr, coin, from_json, Response};

const JUNO: &str = "ujuno";
const IPFS_LINK: &str =
    "https://gateway.pinata.cloud/ipfs/QmQSXMeJRyodyVESWVXT8gd7kQhjrV7sguLnsrXSd6YzvT";

//Test that the contract is instantiated correctly
#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "instantiate"),
            attr("admin", instantiator.to_string())
        ]
    );
}

//Test that the contract can be migrated
#[test]
fn migrate_works() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    //migrate
    let msg = MigrateMsg {};
    let _res: Response = migrate(deps.as_mut(), mock_env(), msg).unwrap();
}

//Test that a listing can be created and then queried
#[test]
fn test_execute_create_listing_valid() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify the response attributes
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "create_post"),
            attr("post_id", "1"),
            attr("seller", listing_creator.clone()),
        ]
    );

    // Query the listing to verify it was created correctly
    let msg = QueryMsg::Listing { listing_id: 1 };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();

    let listing = res.listing.unwrap();
    assert_eq!(listing.listing_title, "Vintage Camera");
    assert_eq!(listing.seller, listing_creator.to_string());
    assert_eq!(listing.price, 100_000_000);
    assert!(!listing.bought);
    assert!(!listing.shipped);
    assert!(!listing.received);
    assert!(!listing.arbitration_requested);
    assert_eq!(listing.buyer, None);
}

//instantiate, create a listing, have a different address purchase it, then query to ensure the listing state changes
#[test]
fn test_purchase_flow() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let listing_buyer = deps.api.addr_make("listing_buyer");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Purchase listing with different address
    let info = message_info(&listing_buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify purchase attributes
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "purchase"),
            attr("post_id", "1"),
            attr("buyer", listing_buyer.clone()),
        ]
    );

    // Query the listing to verify purchase state
    let msg = QueryMsg::Listing { listing_id: 1 };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();

    let listing = res.listing.unwrap();
    assert!(listing.bought);
    assert_eq!(listing.buyer, Some(listing_buyer.to_string()));
}

//Test purchase, sign shipped, and sign received
#[test]
fn test_purchase_sign_shipped_sign_received() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let listing_buyer = deps.api.addr_make("listing_buyer");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //Buyer purchases listing
    let info = message_info(&listing_buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Seller signs shipped
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::SignShipped { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Buyer signs received
    let info = message_info(&listing_buyer, &[]);
    let msg = ExecuteMsg::SignReceived { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify that profiles don't exist
    let msg = QueryMsg::Profile {
        address: listing_creator.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
    let res: ProfileResponse = from_json(&bin).unwrap();
    assert!(res.profile.is_none());

    let msg = QueryMsg::Profile {
        address: listing_buyer.to_string(),
    };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ProfileResponse = from_json(&bin).unwrap();
    assert!(res.profile.is_none());
}

//Test Canceling a purchase
#[test]
fn test_cancel_purchase() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let listing_buyer = deps.api.addr_make("listing_buyer");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Purchase listing with different address
    let info = message_info(&listing_buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Cancel purchase
    let info = message_info(&listing_buyer, &[]);
    let msg = ExecuteMsg::CancelPurchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Verify that the purchase was canceled
    let msg = QueryMsg::Listing { listing_id: 1 };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();
    let listing = res.listing.unwrap();
    assert!(!listing.bought);
    assert!(!listing.shipped);
    assert!(!listing.received);
    assert!(!listing.arbitration_requested);
    assert_eq!(listing.buyer, None);
}

#[test]
//Test that a listing can be edited and then queried
fn test_execute_edit_post_valid() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");

    // Create listing
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Test that the listing can be edited
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::EditListing {
        listing_id: 1,
        external_id: IPFS_LINK.to_string(),
        text: "EDITED: Vintage camera in mint condition, includes original case".to_string(), //Edited text
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
            "Case".to_string(),
        ], //Added case tag
        price: 120_000_000, // Increased price to 120 JUNO
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify edit response
    assert_eq!(
        res.attributes,
        vec![attr("action", "edit_post"), attr("post_id", "1"),]
    );

    // Query the listing to verify changes
    let msg = QueryMsg::Listing { listing_id: 1 };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();

    let listing = res.listing.unwrap();
    assert_eq!(
        listing.text,
        "EDITED: Vintage camera in mint condition, includes original case"
    );
    assert_eq!(listing.price, 120_000_000);
    assert_eq!(listing.tags.len(), 4);
    assert!(listing.tags.contains(&"Case".to_string()));
    assert!(listing.last_edit_date.is_some());
}
#[test]
fn test_execute_edit_post_invalid() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let fake_creator = deps.api.addr_make("fake_creator");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Test that the listing cannot be edited by someone other than the creator
    let info = message_info(&fake_creator, &[]);
    let msg = ExecuteMsg::EditListing {
        listing_id: 1,
        external_id: IPFS_LINK.to_string(),
        text: "EDITED: Vintage camera in mint condition, includes original case".to_string(), //Edited text
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
            "Case".to_string(),
        ], //Added case tag
        price: 120_000_000, // Increased price to 120 JUNO
    };
    let _err = execute(deps.as_mut(), env, info, msg).unwrap_err();
}
#[test]
fn test_execute_delete_post_valid() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //Test that the creator can delete the listing
    let msg = ExecuteMsg::DeleteListing { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Verify that the listing was deleted
    let msg = QueryMsg::Listing { listing_id: 1 };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();
    assert!(res.listing.is_none());
}
#[test]
fn test_execute_delete_post_invalid() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let fake_creator = deps.api.addr_make("fake_creator");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //Test that someone other than the creator cannot delete the listing
    let info = message_info(&fake_creator, &[]);
    let msg = ExecuteMsg::DeleteListing { listing_id: 1 };
    let _err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
}

#[test]
fn test_execute_arbitrate_post_valid() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let listing_buyer = deps.api.addr_make("listing_buyer");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //Buyer purchases listing
    let info = message_info(&listing_buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Seller signs shipped
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::SignShipped { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Buyer requests arbitration
    let info = message_info(&listing_buyer, &[]);
    let msg = ExecuteMsg::RequestArbitration { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Arbiter arbitrates and returns funds to buyer
    let info = message_info(&instantiator, &[]);
    let msg = ExecuteMsg::Arbitrate {
        listing_id: 1,
        funds_recipient: listing_buyer.to_string(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Verify that the listing was deleted
    let msg = QueryMsg::Listing { listing_id: 1 };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();
    assert!(res.listing.is_none());
}

#[test]
fn test_execute_arbitrate_post_invalid() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    //print cosmwasm instantiator address to change in contract.rs for tests to pass
    println!("Instantiator address: {}", instantiator);
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let listing_buyer = deps.api.addr_make("listing_buyer");
    let random_address = deps.api.addr_make("random_address");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //Buyer purchases listing
    let info = message_info(&listing_buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Seller signs shipped
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::SignShipped { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Buyer requests arbitration
    let info = message_info(&listing_buyer, &[]);
    let msg = ExecuteMsg::RequestArbitration { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Arbiter arbitrates and attempts to send funds to random address
    let info = message_info(&instantiator, &[]);
    let msg = ExecuteMsg::Arbitrate {
        listing_id: 1,
        funds_recipient: random_address.to_string(),
    };
    let _err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(
        _err.to_string(),
        "Funds recipient must be the seller or buyer"
    );
}

//Begin Query Tests
#[test]
fn test_query_all_listings() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");

    // Create listing
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Create a second listing
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera 2".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Query all listings
    let msg = QueryMsg::AllListings {
        limit: None,
        //pagination
        start_after: Some(2),
    };
    let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
    let res: AllListingsResponse = from_json(&bin).unwrap();
    //checks descending order
    assert_eq!(res.listings.len(), 1);
    let msg = QueryMsg::AllListings {
        limit: None,
        start_after: None,
    };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: AllListingsResponse = from_json(&bin).unwrap();
    println!("{:?}", res);
}
#[test]
fn test_query_listing() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");

    // Create listing
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //query post
    let msg = QueryMsg::Listing { listing_id: 1 };
    let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();
    assert!(res.listing.is_some());

    //query nonexistent post
    let msg = QueryMsg::Listing { listing_id: 78476 };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ListingResponse = from_json(&bin).unwrap();
    assert!(res.listing.is_none());
}
#[test]
fn test_query_article_count() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");

    // Create listing
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    //query article count
    let msg = QueryMsg::ListingCount {};
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let _res: ListingCountResponse = from_json(&bin).unwrap();
}
#[test]
fn test_query_arbitration_listings() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");
    let listing_buyer = deps.api.addr_make("listing_buyer");

    // Create listing with required JUNO payment
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000, // 100 JUNO
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //Buyer purchases listing
    let info = message_info(&listing_buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Seller signs shipped
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::SignShipped { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Buyer requests arbitration
    let info = message_info(&listing_buyer, &[]);
    let msg = ExecuteMsg::RequestArbitration { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    //Query arbitration listings
    let msg = QueryMsg::ArbitrationListings {
        limit: None,
        start_after: None,
    };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ArbitrationListingsResponse = from_json(&bin).unwrap();
    assert_eq!(res.listings.len(), 1);
}

#[test]
fn test_query_listings_by_title() {
    //instantiate
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    //create mock addresses
    let listing_creator = deps.api.addr_make("listing_creator");

    // Create first listing with Camera in title
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Vintage Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my vintage camera in excellent condition".to_string(),
        tags: vec![
            "Electronics".to_string(),
            "Camera".to_string(),
            "Vintage".to_string(),
        ],
        contact: "Signal: +1234567890".to_string(),
        price: 100_000_000,
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Create second listing with "Camera" in title
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Digital Camera".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Selling my digital camera".to_string(),
        tags: vec!["Electronics".to_string(), "Camera".to_string()],
        contact: "Signal: +1234567890".to_string(),
        price: 50_000_000,
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Create third listing without Camera in title
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Smartphone".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "New smartphone for sale".to_string(),
        tags: vec!["Electronics".to_string(), "Phone".to_string()],
        contact: "Signal: +1234567890".to_string(),
        price: 75_000_000,
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Query listings with Camera in title
    let msg = QueryMsg::SearchListingsByTitle {
        title: "Camera".to_string(),
        limit: None,
    };
    let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
    let res: SearchListingsResponse = from_json(&bin).unwrap();

    // Verify that only listings with "Camera" in title are returned
    assert_eq!(res.listings.len(), 2);
    assert!(res
        .listings
        .iter()
        .all(|listing| listing.listing_title.to_lowercase().contains("camera")));

    // Test case-insensitive search
    let msg = QueryMsg::SearchListingsByTitle {
        title: "camera".to_string(),
        limit: None,
    };
    let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
    let res: SearchListingsResponse = from_json(&bin).unwrap();
    assert_eq!(res.listings.len(), 2);

    // Test with limit
    let msg = QueryMsg::SearchListingsByTitle {
        title: "Camera".to_string(),
        limit: Some(1),
    };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: SearchListingsResponse = from_json(&bin).unwrap();
    assert_eq!(res.listings.len(), 1);
}

#[test]
fn test_cleanup_old_relationships() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let listing_creator = deps.api.addr_make("listing_creator");
    let listing_buyer = deps.api.addr_make("listing_buyer");

    // Create and complete a sale (creates relationship)
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Test Item".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Test description".to_string(),
        tags: vec!["test".to_string()],
        contact: "test@test.com".to_string(),
        price: 100_000_000,
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Purchase listing
    let info = message_info(&listing_buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Sign shipped (creates relationship)
    let info = message_info(&listing_creator, &[]);
    let msg = ExecuteMsg::SignShipped { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Move time forward 31 days
    env.block.time = env.block.time.plus_seconds(2592000 + 86400);

    // Execute cleanup
    let info = message_info(&instantiator, &[]);
    let msg = ExecuteMsg::CleanupOldRelationships {};
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify cleanup results
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "cleanup_old_relationships"),
            attr("relationships_deleted", "1"),
        ]
    );
}

#[test]
fn test_rating_system() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let instantiator = deps.api.addr_make("instantiator");
    let info = message_info(&instantiator, &[]);

    let msg = InstantiateMsg {};
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let seller = deps.api.addr_make("seller");
    let buyer = deps.api.addr_make("buyer");

    // Create profiles for both users
    let info = message_info(&seller, &[]);
    let msg = ExecuteMsg::CreateProfile {
        profile_name: "awesome_seller".to_string(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = message_info(&buyer, &[]);
    let msg = ExecuteMsg::CreateProfile {
        profile_name: "cool_buyer".to_string(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create and complete a transaction
    let info = message_info(&seller, &[]);
    let msg = ExecuteMsg::CreateListing {
        listing_title: "Test Item".to_string(),
        external_id: IPFS_LINK.to_string(),
        text: "Test description".to_string(),
        tags: vec!["test".to_string()],
        contact: "test@test.com".to_string(),
        price: 100_000_000,
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Buyer purchases the item
    let info = message_info(&buyer, &[coin(100_000_000, JUNO)]);
    let msg = ExecuteMsg::Purchase { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Seller marks as shipped
    let info = message_info(&seller, &[]);
    let msg = ExecuteMsg::SignShipped { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Buyer marks as received
    let info = message_info(&buyer, &[]);
    let msg = ExecuteMsg::SignReceived { listing_id: 1 };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Seller rates buyer 5 stars
    let info = message_info(&seller, &[]);
    let msg = ExecuteMsg::RateUser {
        recipient_address: buyer.to_string(),
        rating: 5,
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Buyer rates seller 4 stars
    let info = message_info(&buyer, &[]);
    let msg = ExecuteMsg::RateUser {
        recipient_address: seller.to_string(),
        rating: 4,
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Query profiles and verify ratings
    let msg = QueryMsg::Profile {
        address: seller.to_string(),
    };
    let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
    let res: ProfileResponse = from_json(&bin).unwrap();
    let seller_profile = res.profile.unwrap();

    assert_eq!(seller_profile.transaction_count, 1);
    assert_eq!(seller_profile.ratings, 1);
    assert_eq!(seller_profile.rating_count, 4);
    assert_eq!(seller_profile.average_rating, 4);

    let msg = QueryMsg::Profile {
        address: buyer.to_string(),
    };
    let bin = query(deps.as_ref(), env, msg).unwrap();
    let res: ProfileResponse = from_json(&bin).unwrap();
    let buyer_profile = res.profile.unwrap();

    assert_eq!(buyer_profile.transaction_count, 1);
    assert_eq!(buyer_profile.ratings, 1);
    assert_eq!(buyer_profile.rating_count, 5);
    assert_eq!(buyer_profile.average_rating, 5);
}
