# Changelog

## [0.2.1](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.2.0...v0.2.1) (2025-03-23)


### Bug Fixes

* soft depricate CustomResult variants to supress warnings ([#57](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/57)) ([069b7a4](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/069b7a4a3f31ffe75359f0ba729fbe9f7dfa18d1))

## [0.2.0](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.11...v0.2.0) (2025-03-22)


### ⚠ BREAKING CHANGES

* improve generated schema, eliminate deprecated methods ([#53](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/53))

### Features

* add associated methods to structs for static values ([#56](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/56)) ([f5a97a3](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/f5a97a3aabf7ddcb18813decd854bed63eec0227))
* improve generated schema, eliminate deprecated methods ([#53](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/53)) ([604de64](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/604de64207d04896a2bbafb6d771b98603a3988d))

## [0.1.11](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.10...v0.1.11) (2025-03-18)


### Features

* add new utility functions for CallToolResultContentItem ([#46](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/46)) ([1fe212c](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/1fe212c38a37033180644d938f38e990126465ea))
* add tool_name() method to CallToolRequest ([#52](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/52)) ([2489d90](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/2489d900acee25efdfbc21066110a59665ab2e7f))
* implement default trait for eligible types ([#51](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/51)) ([92022da](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/92022da588dcaa882debeea1c9ca6c5012f5077f))


### Bug Fixes

* updated release action to keep the readme version updated ([#49](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/49)) ([e8b03cf](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/e8b03cfd8879074b8f0ce35860647782549c190b))

## [0.1.10](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.9...v0.1.10) (2025-03-08)


### Bug Fixes

* custom result deserialization conflic with rust_mcp_schema::Result ([#44](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/44)) ([f141060](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/f14106047ee6fdc499f0915ea2029954cf06d634))

## [0.1.9](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.8...v0.1.9) (2025-03-02)


### Features

* introduce CallToolError ([#39](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/39)) ([7b8672d](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/7b8672d7aecd67448bff7d9f3fa933d25ea845bc))
* new cunstructor and conversion utilities for call tool operations ([#41](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/41)) ([0b8f622](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/0b8f6223ca5fd709ab55f4d7f0f9aef6e81e21b0))


### Bug Fixes

* add missing MCPMessage trait for MessageFromClient ([#43](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/43)) ([48dc8af](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/48dc8af9f677d1675e34bb429d7f493d163d51b6))

## [0.1.8](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.7...v0.1.8) (2025-02-23)


### Features

* add SdkError codes and types ([#37](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/37)) ([034ee8f](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/034ee8f31f86314ff879174b33f41924da5cdb72))
* add utility function for easy detection of initialized notifications ([#38](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/38)) ([39400b6](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/39400b6f13b07a0a59180dcba38cc07249e907f7))
* add utility functions for simplified type conversions ([#33](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/33)) ([7447800](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/74478006769feb8692bf6a62cf51c549eb69863b))
* more type conversion utilities ([#36](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/36)) ([9a0abb9](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/9a0abb9a37cd7feb7555a7f98f23ad6a05c7410e))
* new TryFrom implementation for all standard mcp message variants ([#35](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/35)) ([08854f0](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/08854f07f93da8fe2bcd56bab7c910ac490413d8))

## [0.1.7](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.6...v0.1.7) (2025-02-21)


### Features

* add message_type to MCPMessage trait ([#26](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/26)) ([aca2336](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/aca2336f6fa9258098d934bb5e5205ae12ebed1f))
* implement ToMessage trait ([#31](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/31)) ([435f18b](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/435f18b376db0f368f8995fc5c76f8b95eb75ebe))
* introduce FromMessage trait ([#30](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/30)) ([cc46100](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/cc46100a3e66889f21df919c98abefd4598dfa30))

## [0.1.6](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.5...v0.1.6) (2025-02-17)


### Features

* implement new utility functions ([#24](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/24)) ([859b5db](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/859b5dbf4705774dd3f73f50f870aaa573ba624b))


### Bug Fixes

* implemented Error trait for JsonrpcErrorError ([#22](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/22)) ([753bd87](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/753bd87d7b8ccf36a8ca697f7c6c6dacb632a59e))
* serializations to skip None Params ([#25](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/25)) ([1f67654](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/1f67654a3a755d06a5b7dda1577d6763f4315cd0))

## [0.1.5](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.4...v0.1.5) (2025-02-15)


### Features

* implement builder pattern for JsonrpcErrorError ([#18](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/18)) ([71e63e5](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/71e63e51e01fc934e6388b32c50a46602899fe5d))


### Bug Fixes

* Standardize error types  to conform to JSON-RPC ([#20](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/20)) ([47fd818](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/47fd818867626fe318b410e3adfa4b378c51ce69))

## [0.1.4](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.3...v0.1.4) (2025-02-12)


### Features

* enhance `schema_utils` and `mcp_schema` ([#12](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/12)) ([2dbd271](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/2dbd2714259fb4d31927705565a3a25a3c9e89c0))

## [0.1.3](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.2...v0.1.3) (2025-02-10)


### Bug Fixes

* generated schema and schema_utils ([#10](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/10)) ([c077d75](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/c077d7583f9278622c489d95a20afccca2c9982e))

## [0.1.2](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.1...v0.1.2) (2025-02-09)


### Features

* improve schema utils ([#8](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/8)) ([de51f57](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/de51f57461d12294a330c8f0ec432a6dbc50fcca))

## [0.1.1](https://github.com/rust-mcp-stack/rust-mcp-schema/compare/v0.1.0...v0.1.1) (2025-02-09)


### Bug Fixes

* update cargo toml keywords ([#4](https://github.com/rust-mcp-stack/rust-mcp-schema/issues/4)) ([1b75d86](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/1b75d86ec46c91f398e8265f069f642d59e9ec0e))

## 0.1.0 (2025-02-08)


### Features

* Initial release v0.1.0 ([1a77959](https://github.com/rust-mcp-stack/rust-mcp-schema/commit/1a7795923fac8dca1991a47f161369b30ca382fe))
