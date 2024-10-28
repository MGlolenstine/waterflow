use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("Flume error occured! Check that the pipe isn't dropped too early."))]
    Flume,
    #[snafu(display("IO error occured! {e}"))]
    Io { e: std::io::Error },
    #[snafu(display("Bash execution failed! {e}"))]
    Bash { e: String },

    #[cfg(feature = "web")]
    #[snafu(display("WebRequest execution failed! {e}"))]
    WebRequest { e: Box<ureq::Error> },

    #[cfg(feature = "wasm")]
    #[snafu(display("Wasm execution failed! {e}"))]
    Wasm { e: Box<wasmtime::Error> },

    #[cfg(feature = "wasm")]
    #[snafu(display("Wasm memory access error! {e}"))]
    WasmMemoryAccess { e: Box<wasmtime::MemoryAccessError> },

    #[snafu(display("An error occured while trying to parse packets: {e}"))]
    ByparParse { e: bypar::error::Error },

    #[snafu(display("WASM module returned the wrong type"))]
    WasmWrongTypeReturned,
}

impl From<flume::RecvError> for Error {
    fn from(_: flume::RecvError) -> Self {
        Error::Flume
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io { e: value }
    }
}

#[cfg(feature = "web")]
impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Error::WebRequest { e: Box::new(value) }
    }
}

#[cfg(feature = "wasm")]
impl From<wasmtime::Error> for Error {
    fn from(value: wasmtime::Error) -> Self {
        Error::Wasm { e: Box::new(value) }
    }
}

#[cfg(feature = "wasm")]
impl From<wasmtime::MemoryAccessError> for Error {
    fn from(value: wasmtime::MemoryAccessError) -> Self {
        Error::WasmMemoryAccess { e: Box::new(value) }
    }
}

impl From<bypar::error::Error> for Error {
    fn from(value: bypar::error::Error) -> Self {
        Error::ByparParse { e: value }
    }
}
