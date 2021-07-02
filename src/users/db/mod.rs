pub mod couriers;
pub mod admins;
pub mod curators;
pub mod restaurants;
pub use self::restaurants::*;
pub use self::admins::*;
pub use self::curators::*;
pub use self::couriers::*;

use data_encoding::BASE64;
use ring::digest::{Context, SHA256};
use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize,Clone)]
pub struct AuthData {
    pub phone: String,
    pub password: String,
}

pub fn make_hash(password: &str) -> String {
    let mut context = Context::new(&SHA256); 
    context.update(password.as_bytes());
    let pass_hash = context.finish();
    BASE64.encode(pass_hash.as_ref())
}

