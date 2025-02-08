/// Schema Version : Draft
#[cfg(feature = "draft")]
#[path = "generated_schema/draft/mcp_schema.rs"]
mod schema_draft;
#[cfg(feature = "draft")]
pub use schema_draft::*;

#[cfg(all(feature = "schema_utils", feature = "draft"))]
#[path = "generated_schema/draft/schema_utils.rs"]
pub mod schema_utils;

/// Schema Version : 2024_11_05
#[cfg(feature = "2024_11_05")]
#[cfg(not(feature = "draft"))]
#[path = "generated_schema/2024_11_05/mcp_schema.rs"]
mod schema_2024_11_05;

#[cfg(feature = "2024_11_05")]
#[cfg(not(feature = "draft"))]
pub use schema_2024_11_05::*;

#[cfg(all(feature = "schema_utils", feature = "2024_11_05"))]
#[cfg(not(feature = "draft"))]
#[path = "generated_schema/2024_11_05/schema_utils.rs"]
pub mod schema_utils;
