#[cfg(feature = "http-client-simple")]
mod simple;

#[cfg(feature = "http-client-simple")]
pub use simple::{*};
#[cfg(feature = "http-client-reqwest")]
mod reqwest;

#[cfg(feature = "http-client-reqwest")]
pub use reqwest::{*};