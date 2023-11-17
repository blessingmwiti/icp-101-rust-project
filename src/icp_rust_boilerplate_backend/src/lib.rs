#[macro_use]
extern crate serde;
use candid::{CandidType, Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct Product {
    id: u64,
    name: String,
    origin: String,
    // Add other relevant fields
}

impl Storable for Product {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Product {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static PRODUCT_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter for products")
    );

    static PRODUCT_STORAGE: RefCell<StableBTreeMap<u64, Product, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(CandidType, Serialize, Deserialize, Default)]
struct ProductPayload {
    name: String,
    origin: String,
}

#[ic_cdk::query]
fn get_product(id: u64) -> Result<Product, Error> {
    match _get_product(&id) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("A product with id={} not found", id),
        }),
    }
}

fn _get_product(id: &u64) -> Option<Product> {
    PRODUCT_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn add_product(product_payload: ProductPayload) -> Option<Product> {
    let id = PRODUCT_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment product id counter");
    let product = Product {
        id,
        name: product_payload.name,
        origin: product_payload.origin,
    };
    do_insert_product(&product);
    Some(product)
}

fn do_insert_product(product: &Product) {
    PRODUCT_STORAGE.with(|service| service.borrow_mut().insert(product.id, product.clone()));
}

#[ic_cdk::update]
fn update_product(id: u64, payload: ProductPayload) -> Result<Product, Error> {
    match PRODUCT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut product) => {
            product.name = payload.name;
            product.origin = payload.origin;
            do_insert_product(&product);
            Ok(product)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a product with id={}. Product not found", id),
        }),
    }
}

#[ic_cdk::update]
fn delete_product(id: u64) -> Result<Product, Error> {
    match PRODUCT_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a product with id={}. Product not found", id),
        }),
    }
}

#[derive(CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}


ic_cdk::export_candid!();
