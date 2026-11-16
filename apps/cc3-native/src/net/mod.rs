//! Connection to the CC3 server: local server process, bearer bootstrap, WebSocket RPC.
//! Wire format captured from the web client (Effect RPC, JSON), see docs/native/README.md.
pub mod auth;
pub mod rpc;
pub mod server;

pub use rpc::{RpcClient, RpcError};
