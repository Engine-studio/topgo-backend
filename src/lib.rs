#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate actix_web_dev;
pub mod users;
pub mod schema;
pub mod enum_types;
pub mod form;
pub mod temp;
pub mod ordering;
pub mod reports;
