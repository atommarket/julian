use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Listing {
    //tracks specific listings through unique identifier
    pub listing_id: u64,
    //title for FE searches
    pub listing_title: String,
    //ipfs link
    pub external_id: String,
    //price of item
    pub price: u64,
    //store summary of listing / edits
    pub text: String,
    pub tags: Vec<String>,
    pub seller: String,
    //Signal or Session contact
    pub contact: String,
    //If true, item is unbuyable
    pub bought: bool,
    //stores buyer address to ensure signer is legit buyer
    pub buyer: Option<String>,
    //seller marks shipped which is one key to releasing funds
    pub shipped: bool,
    //buyer marks received which is one key to releasing funds
    pub received: bool,
    //arbitration request
    pub arbitration_requested: bool,
    pub creation_date: String,
    pub last_edit_date: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LISTING: Map<u64, Listing> = Map::new("listing");
pub const LAST_LISTING_ID: Item<u64> = Item::new("last_listing_id");
pub const LISTING_COUNT: Item<u64> = Item::new("number_of_listings");
pub const LISTING_TITLES: Map<String, u64> = Map::new("listing_titles");
