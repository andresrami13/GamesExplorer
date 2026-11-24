//! Metadatos públicos de la Steam Store API (`appdetails`), con caché en disco
//! y un limitador que respeta los topes de peticiones.

mod rate_limit;
mod store_api;

pub use rate_limit::RateLimiter;
pub use store_api::{StoreClient, StoreInfo};
