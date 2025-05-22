/// Schema Version: 2024_11_05
#[cfg(feature = "2024_11_05")]
#[path = "generated_schema/2024_11_05/mcp_schema.rs"]
mod mcp_schema;

#[cfg(feature = "2024_11_05")]
pub use mcp_schema::*;

#[cfg(all(feature = "schema_utils", feature = "2024_11_05"))]
#[path = "generated_schema/2024_11_05/schema_utils.rs"]
pub mod schema_utils;

/// Schema Version: 2025_03_26
#[cfg(feature = "2025_03_26")]
#[path = "generated_schema/2025_03_26/mcp_schema.rs"]
mod mcp_schema;

#[cfg(feature = "2025_03_26")]
pub use mcp_schema::*;

#[cfg(all(feature = "schema_utils", feature = "2025_03_26"))]
#[path = "generated_schema/2025_03_26/schema_utils.rs"]
pub mod schema_utils;

/// Schema Version: draft
#[cfg(feature = "draft")]
#[path = "generated_schema/draft/mcp_schema.rs"]
mod mcp_schema;

#[cfg(feature = "draft")]
pub use mcp_schema::*;

#[cfg(all(feature = "schema_utils", feature = "draft"))]
#[path = "generated_schema/draft/schema_utils.rs"]
pub mod schema_utils;
