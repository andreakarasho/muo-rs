//! The network layer: transport, framing, encryption, Huffman, throttles and bans.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Replaces `CancellationToken`: set to true on shutdown.
pub type CancelToken = Arc<AtomicBool>;

pub mod bans;
pub mod client_version;
pub mod compression;
pub mod connection_filters;
pub mod encryption;
pub mod foreign_protocol;
pub mod ip_rate_limiter;
pub mod logs;
pub mod md5;
pub mod movement;
pub mod net_state;
pub mod network;
pub mod network_stats;
pub mod packet_handler;
pub mod ping_server;
pub mod protocol_changes;
pub mod server_info;
pub mod socket_helper;
mod twofish;
