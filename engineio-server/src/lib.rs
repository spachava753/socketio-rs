//! Server must support at least polling and websockets to be functional.
//! Client starts out by polling, then upgrading to websockets if server supports it.
//! Server is also in charge of sending ping packets to the client.
//! Currently, the initial release of the project is only targeting V4 of the engineio protocol.

mod transport;
mod engine;

pub use transport::*;
pub use engine::*;