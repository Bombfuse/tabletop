//! Card domain modules (CRUD + domain structs).
//!
//! Each card type lives in its own module/file.

pub mod action;
pub mod armor_modifier;
pub mod attack;
pub mod interaction;
pub mod item;
pub mod level;
pub mod unit;

#[cfg(test)]
pub(crate) mod test_support;
