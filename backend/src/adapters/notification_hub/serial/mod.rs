/// Functionality for serial communication within the notification hub.
pub(crate) mod channels;
pub mod client;
pub(crate) mod client_pipe;
pub(crate) mod message;

pub use client::SerialClient;
pub use client_pipe::PipeClient;
