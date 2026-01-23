#[path = "common/common.rs"]
pub mod common;
use rust_mcp_schema::ProtocolVersion;

#[test]
fn latest_is_one_before_draft() {
    // Get all supported versions including Draft
    let versions = ProtocolVersion::supported_versions(true);

    // Ensure Draft is last (sanity check)
    let draft_index = versions
        .iter()
        .position(|v| *v == ProtocolVersion::Draft)
        .expect("Draft must exist in supported_versions");

    assert!(draft_index >= 1, "Draft must not be the first version");

    // The version immediately before Draft should be latest
    let expected_latest = &versions[draft_index - 1];

    assert_eq!(
        &ProtocolVersion::latest(),
        expected_latest,
        "`latest()` must return the protocol version immediately before Draft"
    );
}

#[test]
fn supported_versions_are_in_ascending_order() {
    // Get all supported versions, including Draft
    let versions = ProtocolVersion::supported_versions(true);

    // Iterate pairwise and assert each version is less than the next
    for i in 0..versions.len() - 1 {
        let current = &versions[i];
        let next = &versions[i + 1];

        assert!(
            current < next,
            "ProtocolVersion order is incorrect: {:?} is not less than {:?}",
            current,
            next
        );
    }
}
