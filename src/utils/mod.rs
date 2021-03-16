pub mod constants;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
	UnixSocket,
	InetSocket,
}
