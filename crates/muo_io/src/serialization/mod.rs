//! Serialization: the parts independent of Item and Mobile.

pub mod adhoc;
pub mod index;
pub mod persistence;
pub mod reader;
pub mod save_reader;
pub mod save_writer;
pub mod worker;
pub mod writer;

pub use reader::{GenericReader, Guid, TypeRef};
pub use save_reader::{SaveReader, TypesDb};
pub use save_writer::SaveWriter;
pub use worker::SaveWorker;
pub use writer::{GenericSerializable, GenericWriter};
