/// WebSocket connection manager for real-time transaction monitoring
pub mod manager;

pub use manager::{derive_websocket_url_from_rpc, WebSocketManager};
