macro_rules! define_schema_version {
    (
        $feature:literal,
        $mod_name:ident,
        $schema_path:literal,
        $utils_path:literal,
        $validators_path:literal,
        $schema_mod:ident,
        $utils_mod:ident
    ) => {
        #[cfg(feature = $feature)]
        #[path = $schema_path]
        mod $schema_mod;

        #[cfg(all(feature = "schema_utils", feature = $feature))]
        #[path = $utils_path]
        mod $utils_mod;

        #[path = $validators_path]
        mod validators;

        #[cfg(feature = $feature)]
        pub mod $mod_name {
            pub use super::$schema_mod::*;

            #[cfg(feature = "schema_utils")]
            pub mod schema_utils {
                pub use super::super::$utils_mod::*;
            }
        }
    };
}

/// Latest MCP Protocol 2025_06_18
#[cfg(feature = "2025_06_18")]
pub use mcp_2025_06_18::*;

#[cfg(feature = "2025_06_18")]
define_schema_version!(
    "2025_06_18",
    mcp_2025_06_18,
    "generated_schema/2025_06_18/mcp_schema.rs",
    "generated_schema/2025_06_18/schema_utils.rs",
    "generated_schema/2025_06_18/validators.rs",
    __int_2025_06_18,
    __int_utils_2025_06_18
);

#[cfg(feature = "2025_03_26")]
define_schema_version!(
    "2025_03_26",
    mcp_2025_03_26,
    "generated_schema/2025_03_26/mcp_schema.rs",
    "generated_schema/2025_03_26/schema_utils.rs",
    "generated_schema/2025_03_26/validators.rs",
    __int_2025_03_26,
    __int_utils_2025_03_26
);

#[cfg(feature = "2024_11_05")]
define_schema_version!(
    "2024_11_05",
    mcp_2024_11_05,
    "generated_schema/2024_11_05/mcp_schema.rs",
    "generated_schema/2024_11_05/schema_utils.rs",
    "generated_schema/2024_11_05/validators.rs",
    __int_2024_11_05,
    __int_utils_2024_11_05
);

#[cfg(feature = "draft")]
define_schema_version!(
    "draft",
    mcp_draft,
    "generated_schema/draft/mcp_schema.rs",
    "generated_schema/draft/schema_utils.rs",
    "generated_schema/draft/validators.rs",
    __int_draft,
    __int_utils_draft
);

#[path = "generated_schema/protocol_version.rs"]
mod protocol_version;
pub use protocol_version::*;
