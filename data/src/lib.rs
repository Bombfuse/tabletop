//! Shared data layer for Tabletop.
//!
//! This crate is intended to be used as a library by other crates in the repo.
//! It centralizes:
//! - SQLite connection helpers
//! - Domain structs (cards)
//! - CRUD APIs for cards

pub mod shared;

pub mod db;

pub mod cards;
