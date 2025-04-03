/// Schema Version : 2024_11_05
#[cfg(feature = "2024_11_05")]
#[path = "generated_schema/2024_11_05/mcp_schema.rs"]
mod schema_2024_11_05;
#[cfg(feature = "2024_11_05")]
pub use schema_2024_11_05::*;

#[cfg(all(feature = "schema_utils", feature = "2024_11_05"))]
#[path = "generated_schema/2024_11_05/schema_utils.rs"]
pub mod schema_utils;

/// Schema Version : 2025_03_26
#[cfg(feature = "2025_03_26")]
#[cfg(not(feature = "2024_11_05"))]
#[path = "generated_schema/2025_03_26/mcp_schema.rs"]
mod schema_2025_03_26;

#[cfg(feature = "2025_03_26")]
#[cfg(not(feature = "2024_11_05"))]
pub use schema_2025_03_26::*;

#[cfg(all(feature = "schema_utils", feature = "2025_03_26"))]
#[cfg(not(feature = "2024_11_05"))]
#[path = "generated_schema/2025_03_26/schema_utils.rs"]
pub mod schema_utils;
