#[cfg(any(feature = "2025_11_25", feature = "draft"))]
mod tests_schema_utils {
    use rust_mcp_schema::mcp_2025_11_25::*;
    use serde_json::{json, Value};

    // Helper to extract the inner map from a JSON value
    fn map_from_json(value: Value) -> serde_json::Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn test_string_schema_minimal() {
        let map = map_from_json(json!({
            "type": "string"
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        match result {
            PrimitiveSchemaDefinition::StringSchema(s) => {
                assert_eq!(s.type_(), "string");
                assert!(s.default.is_none());
                assert!(s.title.is_none());
                assert!(s.description.is_none());
                assert!(s.max_length.is_none());
                assert!(s.min_length.is_none());
                assert!(s.format.is_none());
            }
            _ => panic!("Expected StringSchema"),
        }
    }

    #[test]
    fn test_string_schema_full() {
        let map = map_from_json(json!({
            "type": "string",
            "title": "Username",
            "description": "User's username",
            "default": "guest",
            "minLength": 3,
            "maxLength": 20,
            "format": "byte"
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        if let PrimitiveSchemaDefinition::StringSchema(s) = result {
            assert_eq!(s.type_(), "string");
            assert_eq!(s.title.as_deref(), Some("Username"));
            assert_eq!(s.description.as_deref(), Some("User's username"));
            assert_eq!(s.default.as_deref(), Some("guest"));
            assert_eq!(s.min_length, Some(3));
            assert_eq!(s.max_length, Some(20));
            #[cfg(not(any(feature = "2025_11_25", feature = "draft")))]
            assert_eq!(s.format, Some(StringSchemaFormat::Byte));
        } else {
            panic!("Expected StringSchema");
        }
    }

    #[test]
    fn test_number_schema_minimal() {
        let map = map_from_json(json!({
            "type": "integer"
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        match result {
            PrimitiveSchemaDefinition::NumberSchema(n) => {
                assert_eq!(n.type_, NumberSchemaType::Integer);
            }
            _ => panic!("Expected NumberSchema"),
        }
    }

    #[test]
    fn test_number_schema_full() {
        let map = map_from_json(json!({
            "type": "number",
            "title": "Age",
            "description": "Age in years",
            "minimum": 0,
            "maximum": 130,
            "default": 25
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        if let PrimitiveSchemaDefinition::NumberSchema(n) = result {
            assert_eq!(n.type_, NumberSchemaType::Number);
            assert_eq!(n.title.as_deref(), Some("Age"));
            assert_eq!(n.minimum, Some(0));
            assert_eq!(n.maximum, Some(130));
            assert_eq!(n.default, Some(25));
        } else {
            panic!("Expected NumberSchema");
        }
    }

    #[test]
    fn test_boolean_schema() {
        let map = map_from_json(json!({
            "type": "boolean",
            "title": "Is Active",
            "default": true
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        if let PrimitiveSchemaDefinition::BooleanSchema(b) = result {
            assert_eq!(b.type_(), "boolean");
            assert_eq!(b.default, Some(true));
            assert_eq!(b.title.as_deref(), Some("Is Active"));
        } else {
            panic!("Expected BooleanSchema");
        }
    }

    #[test]
    fn test_untitled_single_select_enum_schema() {
        let map = map_from_json(json!({
            "type": "string",
            "enum": ["red", "green", "blue"],
            "default": "green"
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        match result {
            PrimitiveSchemaDefinition::UntitledSingleSelectEnumSchema(e) => {
                assert_eq!(e.type_(), "string");
                assert_eq!(e.enum_, vec!["red", "green", "blue"]);
                assert_eq!(e.default.as_deref(), Some("green"));
            }
            _ => panic!("Expected UntitledSingleSelectEnumSchema"),
        }
    }

    #[test]
    fn test_titled_single_select_enum_schema() {
        let map = map_from_json(json!({
            "type": "string",
            "oneOf": [
                { "const": "admin", "title": "Administrator" },
                { "const": "user",  "title": "Regular User" }
            ],
            "default": "user"
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        match result {
            PrimitiveSchemaDefinition::TitledSingleSelectEnumSchema(e) => {
                assert_eq!(e.one_of.len(), 2);
                assert_eq!(e.one_of[0].const_, "admin");
                assert_eq!(e.one_of[0].title, "Administrator");
                assert_eq!(e.default.as_deref(), Some("user"));
            }
            _ => panic!("Expected TitledSingleSelectEnumSchema"),
        }
    }

    #[test]
    fn test_untitled_multi_select_enum_schema() {
        let map = map_from_json(json!({
            "type": "array",
            "items": { "type": "string", "enum": ["read", "write", "delete"] },
            "minItems": 1,
            "maxItems": 3,
            "default": ["read"]
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        match result {
            PrimitiveSchemaDefinition::UntitledMultiSelectEnumSchema(e) => {
                assert_eq!(e.min_items, Some(1));
                assert_eq!(e.max_items, Some(3));
                assert_eq!(e.default, vec!["read"]);
                let UntitledMultiSelectEnumSchemaItems { enum_, .. } = &e.items;
                assert_eq!(*enum_, vec!["read", "write", "delete"]);
            }
            _ => panic!("Expected UntitledMultiSelectEnumSchema"),
        }
    }

    #[test]
    fn test_titled_multi_select_enum_schema() {
        let map = map_from_json(json!({
            "type": "array",
            "items": {
                "anyOf": [
                    { "const": "admin", "title": "Admin Role" },
                    { "const": "editor", "title": "Editor" }
                ]
            },
            "default": ["editor"]
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();

        match result {
            PrimitiveSchemaDefinition::TitledMultiSelectEnumSchema(e) => {
                assert_eq!(e.default, vec!["editor"]);
                let TitledMultiSelectEnumSchemaItems { any_of } = &e.items;
                assert_eq!(any_of.len(), 2);
                assert_eq!(any_of[0].const_, "admin");
                assert_eq!(any_of[0].title, "Admin Role");
            }
            _ => panic!("Expected TitledMultiSelectEnumSchema"),
        }
    }

    #[test]
    fn test_legacy_titled_enum_schema() {
        let map = map_from_json(json!({
            "type": "string",
            "enum": ["draft", "published"],
            "enumNames": ["Draft", "Published"],
            "default": "draft",
            "title": "Status"
        }));

        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        match result {
            PrimitiveSchemaDefinition::LegacyTitledEnumSchema(e) => {
                assert_eq!(e.enum_, vec!["draft", "published"]);
                assert_eq!(e.enum_names, vec!["Draft", "Published"]);
                assert_eq!(e.default.as_deref(), Some("draft"));
                assert_eq!(e.title.as_deref(), Some("Status"));
            }
            _ => panic!("Expected LegacyTitledEnumSchema"),
        }
    }

    #[test]
    fn test_invalid_schema_rejected() {
        // Missing type
        let map = map_from_json(json!({
            "title": "Bad"
        }));

        let err = PrimitiveSchemaDefinition::try_from(&map);
        assert!(err.is_err());

        // Wrong type
        let map2 = map_from_json(json!({
            "type": "object"
        }));
        assert!(PrimitiveSchemaDefinition::try_from(&map2).is_err());
    }

    #[test]
    fn test_priority_of_variants() {
        // This tests that more specific schemas are matched before generic ones
        // For example: a schema with "oneOf" should be TitledSingleSelect, not fall into StringSchema

        let titled_single = json!({
            "type": "string",
            "oneOf": [{ "const": "yes", "title": "Yes" }],
            "title": "Confirmed?"
        });

        let map = map_from_json(titled_single);
        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        assert!(matches!(result, PrimitiveSchemaDefinition::TitledSingleSelectEnumSchema(_)));

        let untitled_enum = json!({
            "type": "string",
            "enum": ["a", "b"]
        });

        let map = map_from_json(untitled_enum);
        let result = PrimitiveSchemaDefinition::try_from(&map).unwrap();
        assert!(matches!(result, PrimitiveSchemaDefinition::UntitledSingleSelectEnumSchema(_)));
    }
}

#[test]
fn adhoc() {
    use rust_mcp_schema::mcp_2025_11_25::schema_utils::*;
    use rust_mcp_schema::mcp_2025_11_25::*;

    let str = r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{"sampling":{},"elicitation":{},"roots":{"listChanged":true}},"clientInfo":{"name":"inspector-client","version":"0.17.2"}}}"#;

    let msg: ClientMessage = serde_json::from_str(str).unwrap();
    assert!(matches!(
        msg,
        ClientMessage::Request(ClientJsonrpcRequest::InitializeRequest(_))
    ));

    let msg: ClientJsonrpcRequest = serde_json::from_str(str).unwrap();
    assert!(matches!(msg, ClientJsonrpcRequest::InitializeRequest(_)));

    let msg: ClientRequest = serde_json::from_str(str).unwrap();
    assert!(matches!(msg, ClientRequest::InitializeRequest(_)));
}
