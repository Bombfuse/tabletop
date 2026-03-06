//! Shared data layer for Tabletop.
//!
//! This crate is intended to be used as a library by other crates in the repo.
//! It centralizes:
//! - SQLite connection helpers
//! - Domain structs (cards, hex grids)
//! - CRUD APIs for cards and other domain data

pub mod shared;

pub mod db;

pub mod cards;

pub mod hex_grids;
