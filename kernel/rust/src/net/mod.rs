pub mod socket;

pub use socket::{
    socket_accept, socket_bind, socket_close, socket_connect, socket_create,
    socket_has_pending_connections, socket_listen, socket_read, socket_write,
};
