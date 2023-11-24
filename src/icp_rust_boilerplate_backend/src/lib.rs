#[macro_use]
extern crate serde;
use candid::{CandidType, Decode, Encode};
use validator::Validate;
use ic_cdk::api::caller;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct Product {
    id: u64,
    owner: String,
    name: String,
    origin: String
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

#[derive(CandidType, Serialize, Deserialize, Default, Validate)]
struct ProductPayload {
    #[validate(length(min = 3))]
    name: String,
    #[validate(length(min = 2))] // The shortest format of countries' names is the Alpha-2code which is 2 characters
    origin: String,
}

// Function to get a product from the canister
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

// Function to add a product to the canister
#[ic_cdk::update]
fn add_product(product_payload: ProductPayload) -> Result<Product, Error> {
    // Validates payload
    let check_payload = _check_input(&product_payload);
    // Returns an error if validations failed
    if check_payload.is_err(){
        return Err(check_payload.err().unwrap());
    }
    let id = PRODUCT_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment product id counter");
    let product = Product {
        id,
        owner: caller().to_string(),
        name: product_payload.name,
        origin: product_payload.origin,
    };
    // save product
    do_insert_product(&product);
    Ok(product)
}

fn do_insert_product(product: &Product) {
    PRODUCT_STORAGE.with(|service| service.borrow_mut().insert(product.id, product.clone()));
}

// Function to update a product in the canister
#[ic_cdk::update]
fn update_product(id: u64, payload: ProductPayload) -> Result<Product, Error> {
    match PRODUCT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut product) => {
            // Validates whether caller is the owner of the product
            let check_if_owner = _check_if_owner(&product);
            if check_if_owner.is_err() {
                return Err(check_if_owner.err().unwrap())
            }
            // Validates payload
            let check_payload = _check_input(&payload);
            // Returns an error if validations failed
            if check_payload.is_err(){
                return Err(check_payload.err().unwrap());
            }
            product.name = payload.name;
            product.origin = payload.origin;
            // save updated product
            do_insert_product(&product);
            Ok(product)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a product with id={}. Product not found", id),
        }),
    }
}

// Function to delete a product
#[ic_cdk::update]
fn delete_product(id: u64) -> Result<Product, Error> {
    let product = _get_product(&id).expect(&format!("couldn't delete a product with id={}. product not found.", id));
    // Validates whether caller is the owner of the product
    let check_if_owner = _check_if_owner(&product);
    if check_if_owner.is_err() {
        return Err(check_if_owner.err().unwrap())
    }
    match PRODUCT_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a product with id={}. Product not found", id),
        }),
    }
}

// Helper function to check the input data of the payload
fn _check_input(payload: &ProductPayload) -> Result<(), Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error:: ValidationFailed{ content: check_payload.err().unwrap().to_string()})
    }else{
        Ok(())
    }
}

// Helper function to check whether the caller is the owner of a product
fn _check_if_owner(product: &Product) -> Result<(), Error> {
    if product.owner.to_string() != caller().to_string(){
        return Err(Error:: AuthenticationFailed{ msg: format!("Caller={} isn't the owner of the product with id={}", caller(), product.id) })  
    }else{
        Ok(())
    }
}

#[derive(CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    ValidationFailed { content: String},
    AuthenticationFailed{ msg: String}
}


ic_cdk::export_candid!();
