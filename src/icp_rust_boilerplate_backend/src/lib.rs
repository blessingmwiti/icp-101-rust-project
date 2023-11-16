#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct Product {
    id: u64,
    name: String,
    description: String,
    manufacturer: String,
    created_at: u64,
    updated_at: Option<u64>,
    ethical_rating: u8,
    unique_identifier: String, // Unique identifier for anti-counterfeit measures.
}

// Custom error type for handling different error cases.
#[derive(CandidType, Deserialize, Serialize)]
enum SupplyChainError {
    #[serde(rename = "NotFound")]
    NotFound { msg: String },
    #[serde(rename = "Counterfeit")]
    Counterfeit { msg: String },
}

// Struct to represent a supply chain, holding a map of products.
#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct SupplyChain {
    products: BTreeMap<u64, Product>,
}

// Functions to handle supply chain operations.
#[export_name = "canister_query get_product"]
fn get_product(id: u64) -> Result<Product, SupplyChainError> {
    match _get_product(id) {
        Some(product) => Ok(product),
        None => Err(SupplyChainError::NotFound {
            msg: format!("Product with id={} not found", id),
        }),
    }
}

fn _get_product(id: u64) -> Option<Product> {
    let supply_chain: SupplyChain = storage::get().unwrap_or_default();
    supply_chain.products.get(&id).cloned()
}

#[export_name = "canister_update add_product"]
fn add_product(product: Product) -> Option<Product> {
    let mut supply_chain: SupplyChain = storage::get().unwrap_or_default();
    let id = api::id();
    let unique_identifier = generate_unique_identifier();
    let product = Product {
        id,
        unique_identifier,
        created_at: api::time(),
        updated_at: None,
        ethical_rating: product.ethical_rating,
        ..product
    };
    supply_chain.products.insert(id, product.clone());
    storage::stable_save((supply_chain,)).unwrap();
    Some(product)
}

// Function to generate a unique identifier (you may implement this based on your requirements).
fn generate_unique_identifier() -> String {
    // Placeholder implementation, you might want to use a more sophisticated approach.
    format!("UID-{}", api::id())
}

#[export_name = "canister_update verify_product_authenticity"]
fn verify_product_authenticity(id: u64, unique_identifier: String) -> Result<(), SupplyChainError> {
    match _get_product(id) {
        Some(product) => {
            if product.unique_identifier == unique_identifier {
                Ok(())
            } else {
                Err(SupplyChainError::Counterfeit {
                    msg: "Product is counterfeit".to_string(),
                })
            }
        }
        None => Err(SupplyChainError::NotFound {
            msg: format!("Product with id={} not found", id),
        }),
    }
}

#[export_name = "canister_update update_product"]
fn update_product(id: u64, updated_product: Product) -> Result<Product, SupplyChainError> {
    let mut supply_chain: SupplyChain = storage::get().unwrap_or_default();
    match supply_chain.products.get_mut(&id) {
        Some(product) => {
            product.ethical_rating = updated_product.ethical_rating;
            product.updated_at = Some(api::time());
            storage::stable_save((supply_chain,)).unwrap();
            Ok(product.clone())
        }
        None => Err(SupplyChainError::NotFound {
            msg: format!("Product with id={} not found", id),
        }),
    }
}

#[export_name = "canister_update delete_product"]
fn delete_product(id: u64) -> Result<Product, SupplyChainError> {
    let mut supply_chain: SupplyChain = storage::get().unwrap_or_default();
    match supply_chain.products.remove(&id) {
        Some(product) => {
            storage::stable_save((supply_chain,)).unwrap();
            Ok(product)
        }
        None => Err(SupplyChainError::NotFound {
            msg: format!("Product with id={} not found", id),
        }),
    }
}

// Exporting the Candid interface.
export_candid!();
