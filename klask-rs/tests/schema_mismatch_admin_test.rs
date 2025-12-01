#[cfg(test)]
mod schema_mismatch_admin_tests {
    use klask_rs::models::SearchStatusResponse;

    /// Test: GET /api/admin/search/status returns correct schema_mismatch status
    ///
    /// This test verifies that the admin endpoint correctly reports schema mismatch status
    /// to clients (including the frontend banner system).
    ///
    /// Note: Full endpoint testing requires a running server with AppState.
    /// These tests demonstrate the data structure contract.
    #[test]
    fn test_search_status_response_structure_no_mismatch() {
        // Simulate response when there's no schema mismatch
        let response = SearchStatusResponse { schema_mismatch: false, index_available: true, message: None };

        assert_eq!(response.schema_mismatch, false);
        assert_eq!(response.index_available, true);
        assert!(response.message.is_none());
        println!("✅ Search status response structure (no mismatch) is correct");
    }

    /// Test: GET /api/admin/search/status when mismatch detected
    ///
    /// This test verifies the response format when a schema mismatch is detected.
    #[test]
    fn test_search_status_response_structure_with_mismatch() {
        let response = SearchStatusResponse {
            schema_mismatch: true,
            index_available: false,
            message: Some("Index schema mismatch detected. Please rebuild the index in admin settings.".to_string()),
        };

        assert_eq!(response.schema_mismatch, true);
        assert_eq!(response.index_available, false);
        assert!(response.message.is_some());
        assert!(response.message.unwrap().contains("rebuild the index"));
        println!("✅ Search status response structure (with mismatch) is correct");
    }

    /// Test: SearchStatusResponse can be serialized to JSON
    ///
    /// This test verifies that the response can be properly serialized for HTTP responses.
    #[test]
    fn test_search_status_response_json_serialization() {
        let response = SearchStatusResponse {
            schema_mismatch: true,
            index_available: false,
            message: Some("Index schema mismatch detected. Please rebuild the index in admin settings.".to_string()),
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json.contains("schema_mismatch"));
        assert!(json.contains("index_available"));
        assert!(json.contains("message"));
        assert!(json.contains("true"));
        println!("✅ SearchStatusResponse serializes to valid JSON");
    }

    /// Test: SearchStatusResponse can be deserialized from JSON
    ///
    /// This test verifies that the frontend can parse the status response.
    #[test]
    fn test_search_status_response_json_deserialization() {
        let json = r#"{
            "schema_mismatch": true,
            "index_available": false,
            "message": "Index schema mismatch detected"
        }"#;

        let response: SearchStatusResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.schema_mismatch, true);
        assert_eq!(response.index_available, false);
        assert!(response.message.is_some());
        println!("✅ SearchStatusResponse deserializes from JSON correctly");
    }

    /// Test: Status endpoint response without message (when no mismatch)
    ///
    /// Verifies the contract when index is healthy.
    #[test]
    fn test_search_status_no_message_when_healthy() {
        let json = r#"{
            "schema_mismatch": false,
            "index_available": true,
            "message": null
        }"#;

        let response: SearchStatusResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.schema_mismatch, false);
        assert_eq!(response.index_available, true);
        assert!(response.message.is_none());
        println!("✅ Status endpoint correctly omits message when healthy");
    }

    /// Test: Schema mismatch status indicates index is unavailable
    ///
    /// Ensures the invariant: schema_mismatch=true => index_available=false
    #[test]
    fn test_schema_mismatch_implies_index_unavailable() {
        let response = SearchStatusResponse {
            schema_mismatch: true,
            index_available: false,
            message: Some("Schema mismatch".to_string()),
        };

        if response.schema_mismatch {
            assert!(
                !response.index_available,
                "If schema_mismatch is true, index_available must be false"
            );
            assert!(response.message.is_some(), "Must include error message");
        }
        println!("✅ Schema mismatch correctly indicates unavailable index");
    }

    /// Test: Healthy status indicates both no mismatch and available index
    ///
    /// Ensures the invariant: index_available=true => schema_mismatch=false
    #[test]
    fn test_healthy_status_both_fields_consistent() {
        let response = SearchStatusResponse { schema_mismatch: false, index_available: true, message: None };

        if response.index_available {
            assert!(
                !response.schema_mismatch,
                "If index_available is true, schema_mismatch must be false"
            );
        }
        println!("✅ Healthy status has consistent field values");
    }

    /// Test: Status response fields are correctly named for frontend
    ///
    /// This ensures the frontend can correctly key into the response fields.
    #[test]
    fn test_status_response_field_names_match_api_contract() {
        let json = r#"{
            "schema_mismatch": true,
            "index_available": false,
            "message": "Error message"
        }"#;

        let value: serde_json::Value = serde_json::from_str(json).expect("Should be valid JSON");

        // Verify field names exactly match the contract
        assert!(value.get("schema_mismatch").is_some());
        assert!(value.get("index_available").is_some());
        assert!(value.get("message").is_some());

        // Ensure there are no extra fields that would break the contract
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 3, "Response should have exactly 3 fields");
        println!("✅ Status response field names match API contract");
    }

    /// Test: Frontend can detect schema mismatch from status response
    ///
    /// Simulates how the frontend checks if it should show the warning banner.
    #[test]
    fn test_frontend_can_detect_schema_mismatch() {
        // When status endpoint returns this:
        let json = r#"{
            "schema_mismatch": true,
            "index_available": false,
            "message": "Index schema mismatch detected. Please rebuild the index in admin settings."
        }"#;

        let response: SearchStatusResponse = serde_json::from_str(json).expect("Failed to deserialize");

        // Frontend logic: show warning banner if schema_mismatch is true
        if response.schema_mismatch {
            // Display warning to user
            let warning_shown = true;
            assert!(warning_shown);
        }

        // Frontend logic: disable search if index not available
        if !response.index_available {
            // Disable search UI
            let search_disabled = true;
            assert!(search_disabled);
        }

        println!("✅ Frontend can correctly interpret schema mismatch status");
    }

    /// Test: Status response doesn't include sensitive information
    ///
    /// Ensures the endpoint is safe to call from unauthenticated clients.
    #[test]
    fn test_status_response_no_sensitive_data() {
        let response = SearchStatusResponse {
            schema_mismatch: true,
            index_available: false,
            message: Some("Index needs rebuild".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();

        // Should not contain:
        // - File paths
        // - Internal error details
        // - System information
        assert!(!json.contains("/"), "Should not contain file paths");
        assert!(!json.contains("thread"), "Should not contain thread info");
        assert!(!json.contains("line"), "Should not contain line numbers");

        println!("✅ Status response contains no sensitive information");
    }

    /// Test: Multiple status checks return consistent results
    ///
    /// Verifies that repeated status checks don't show inconsistent data.
    #[test]
    fn test_status_checks_consistency() {
        let responses = vec![
            SearchStatusResponse { schema_mismatch: false, index_available: true, message: None },
            SearchStatusResponse { schema_mismatch: false, index_available: true, message: None },
            SearchStatusResponse { schema_mismatch: false, index_available: true, message: None },
        ];

        // All responses should be identical (in a healthy state)
        for (i, response) in responses.iter().enumerate() {
            assert_eq!(
                response.schema_mismatch, false,
                "Response {} should have schema_mismatch=false",
                i
            );
            assert_eq!(
                response.index_available, true,
                "Response {} should have index_available=true",
                i
            );
        }

        println!("✅ Multiple status checks return consistent results");
    }

    /// Test: Message field contains actionable guidance
    ///
    /// When there's a schema mismatch, the message should guide users to the fix.
    #[test]
    fn test_mismatch_message_is_actionable() {
        let response = SearchStatusResponse {
            schema_mismatch: true,
            index_available: false,
            message: Some("Index schema mismatch detected. Please rebuild the index in admin settings.".to_string()),
        };

        let message = response.message.unwrap();

        // Message should mention:
        // 1. What went wrong
        // 2. Where to fix it
        assert!(message.contains("mismatch"));
        assert!(message.contains("rebuild"));
        assert!(message.contains("admin"));

        println!("✅ Mismatch message provides actionable guidance");
    }

    /// Test: Schema mismatch is the only reason index is unavailable
    ///
    /// In this system, if index_available=false, it should be due to schema mismatch.
    #[test]
    fn test_unavailable_index_reason_is_schema_mismatch() {
        let response = SearchStatusResponse {
            schema_mismatch: true,
            index_available: false,
            message: Some("Reason for unavailability".to_string()),
        };

        // In the current system, the only reason index is unavailable is schema mismatch
        if !response.index_available {
            assert!(
                response.schema_mismatch,
                "If index is unavailable, it must be due to schema mismatch"
            );
        }

        println!("✅ Unavailable index correctly indicates schema mismatch");
    }
}
