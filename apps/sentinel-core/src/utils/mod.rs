//! Utility macros used throughout the codebase.
//!
//! - [`impl_repository!`] — generates a full CRUD + pagination repository struct.
//! - [`map_entity!`] — generates `From` impls or named constructors for type-mapping.

#[macro_use]
pub mod map_entity;

#[macro_use]
pub mod impl_repository;
