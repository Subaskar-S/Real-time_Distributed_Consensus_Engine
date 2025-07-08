//! # State
//! 
//! Pluggable state machine interface and implementations.
//! 
//! This module provides the state machine abstraction that allows
//! different backends (in-memory, RocksDB, etc.) to be used with Raft.

pub mod state_machine;
pub mod kv_store;
pub mod error;

#[cfg(feature = "rocksdb-backend")]
pub mod rocksdb_store;

pub use state_machine::StateMachine;
pub use kv_store::InMemoryKvStore;
pub use error::StateError;

#[cfg(feature = "rocksdb-backend")]
pub use rocksdb_store::RocksDbStore;
