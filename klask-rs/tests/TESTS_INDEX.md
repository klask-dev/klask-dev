# Schema Mismatch Tests Index

Quick reference for all 51 tests created for schema mismatch detection and handling.

## Test File 1: search_service_test.rs
### Unit Tests (12 tests, async)

Located in `/workspace/klask-rs/tests/search_service_test.rs` (lines 809-1200)

1. **test_service_initializes_with_no_schema_mismatch_when_index_fresh**
   - Verifies fresh service has no mismatch flag
   - Type: Unit test
   - Async: Yes

2. **test_search_fails_when_schema_mismatch_detected**
   - Verifies search works when no mismatch exists
   - Type: Unit test
   - Async: Yes

3. **test_rebuild_index_clears_schema_mismatch_flag**
   - Verifies rebuild clears the mismatch flag
   - Type: Unit test
   - Async: Yes

4. **test_rebuild_index_clears_all_documents**
   - Verifies rebuild removes all indexed documents
   - Type: Unit test
   - Async: Yes

5. **test_reset_index_behaves_like_rebuild_index**
   - Verifies reset_index is equivalent to rebuild_index
   - Type: Unit test
   - Async: Yes

6. **test_can_search_and_index_after_rebuild**
   - Verifies full functionality after rebuild
   - Type: Unit test
   - Async: Yes

7. **test_rebuild_index_multiple_times**
   - Verifies rebuild is idempotent (can be called multiple times)
   - Type: Unit test
   - Async: Yes

8. **test_has_schema_mismatch_is_thread_safe**
   - Verifies concurrent reads of mismatch flag are safe
   - Type: Unit test (concurrent)
   - Async: Yes

9. **test_concurrent_rebuild_and_schema_mismatch_check**
   - Verifies concurrent rebuild and status checks are safe
   - Type: Unit test (concurrent)
   - Async: Yes

10. **test_schema_mismatch_flag_consistency**
    - Verifies flag returns same value on repeated reads
    - Type: Unit test
    - Async: Yes

11. **test_rebuild_index_preserves_index_directory**
    - Verifies directory is not deleted during rebuild
    - Type: Unit test
    - Async: Yes

12. **test_index_functionality_after_multiple_rebuilds**
    - Verifies full rebuild→index→search cycle works repeatedly
    - Type: Unit test
    - Async: Yes

---

## Test File 2: schema_mismatch_admin_test.rs
### Admin API Tests (13 tests, sync)

Located in `/workspace/klask-rs/tests/schema_mismatch_admin_test.rs` (lines 1-322)

1. **test_search_status_response_structure_no_mismatch**
   - Verifies response format when index is healthy
   - Type: API contract test
   - Async: No

2. **test_search_status_response_structure_with_mismatch**
   - Verifies response format when mismatch detected
   - Type: API contract test
   - Async: No

3. **test_search_status_response_json_serialization**
   - Verifies response can be serialized to JSON
   - Type: Serialization test
   - Async: No

4. **test_search_status_response_json_deserialization**
   - Verifies response can be deserialized from JSON
   - Type: Serialization test
   - Async: No

5. **test_search_status_no_message_when_healthy**
   - Verifies message field is omitted when healthy
   - Type: API contract test
   - Async: No

6. **test_schema_mismatch_implies_index_unavailable**
   - Verifies invariant: mismatch → unavailable
   - Type: Invariant test
   - Async: No

7. **test_healthy_status_both_fields_consistent**
   - Verifies invariant: available → no mismatch
   - Type: Invariant test
   - Async: No

8. **test_status_response_field_names_match_api_contract**
   - Verifies frontend can parse response
   - Type: API contract test
   - Async: No

9. **test_frontend_can_detect_schema_mismatch**
   - Verifies frontend logic flow
   - Type: Integration pattern test
   - Async: No

10. **test_status_response_no_sensitive_data**
    - Verifies endpoint is safe for unauthenticated access
    - Type: Security test
    - Async: No

11. **test_status_checks_consistency**
    - Verifies multiple checks return identical data
    - Type: Consistency test
    - Async: No

12. **test_mismatch_message_is_actionable**
    - Verifies user gets guidance on fixing issue
    - Type: UX test
    - Async: No

13. **test_unavailable_index_reason_is_schema_mismatch**
    - Verifies mismatch is the only unavailability reason
    - Type: Invariant test
    - Async: No

---

## Test File 3: schema_mismatch_integration_test.rs
### Integration Tests (16 tests, async)

Located in `/workspace/klask-rs/tests/schema_mismatch_integration_test.rs` (lines 1-419)

1. **test_admin_status_endpoint_response_format**
   - Verifies endpoint response structure
   - Type: Integration test
   - Async: Yes

2. **test_admin_status_with_mismatch_detected**
   - Verifies endpoint correctly reports mismatch
   - Type: Integration test
   - Async: Yes

3. **test_status_endpoint_no_auth_required**
   - Verifies endpoint accessible without authentication
   - Type: Security test
   - Async: Yes

4. **test_admin_reset_endpoint_clears_mismatch**
   - Verifies reset operation works
   - Type: Integration test
   - Async: Yes

5. **test_admin_reset_response_includes_was_mismatch**
   - Verifies reset response includes field
   - Type: Integration test
   - Async: Yes

6. **test_admin_reset_response_when_no_mismatch**
   - Verifies reset response when no prior mismatch
   - Type: Integration test
   - Async: Yes

7. **test_multiple_reset_calls_idempotent**
   - Verifies reset can be called multiple times
   - Type: Integration test
   - Async: Yes

8. **test_full_workflow_detect_and_reset**
   - End-to-end user workflow test
   - Type: Workflow test
   - Async: Yes

9. **test_concurrent_status_checks_during_reset**
   - Verifies thread safety during active reset
   - Type: Concurrency test
   - Async: Yes

10. **test_status_endpoint_reflects_current_state**
    - Verifies no caching of stale status
    - Type: Integration test
    - Async: Yes

11. **test_status_consistency_in_healthy_state**
    - Verifies consistent data in stable state
    - Type: Consistency test
    - Async: Yes

12. **test_status_consistency_during_recovery**
    - Verifies accuracy during active recovery
    - Type: Consistency test
    - Async: Yes

13. **test_status_not_blocked_by_admin_ops**
    - Verifies status endpoint performance
    - Type: Performance test
    - Async: Yes

14. **test_reset_error_handling**
    - Verifies graceful error handling
    - Type: Error handling test
    - Async: Yes

15. **test_status_after_rebuild**
    - Verifies final state after successful rebuild
    - Type: Integration test
    - Async: Yes

16. **test_mismatch_flag_independence**
    - Verifies flag doesn't affect other operations
    - Type: Isolation test
    - Async: Yes

---

## Test Execution Commands

### Run All Tests
```bash
cd /workspace/klask-rs
cargo test schema_mismatch
```

### Run By File
```bash
# Unit tests
cargo test --test search_service_test schema_mismatch

# Admin API tests
cargo test --test schema_mismatch_admin_test

# Integration tests
cargo test --test schema_mismatch_integration_test
```

### Run Specific Test
```bash
cargo test --test schema_mismatch_admin_test test_search_status_response_structure_no_mismatch
```

### Run with Output
```bash
cargo test schema_mismatch -- --nocapture
```

### Run Single-threaded
```bash
cargo test schema_mismatch -- --test-threads=1
```

---

## Test Statistics

| Category | Count |
|----------|-------|
| Unit Tests (search_service_test.rs) | 12 |
| Admin API Tests (schema_mismatch_admin_test.rs) | 13 |
| Integration Tests (schema_mismatch_integration_test.rs) | 16 |
| **Total** | **51** |

| Type | Count |
|------|-------|
| Async Tests | 28 |
| Sync Tests | 23 |
| **Total** | **51** |

---

## Coverage by Functionality

| Functionality | Tests | Test Count |
|---|---|---|
| Fresh Index Initialization | search_service_test.rs | 1 |
| Search Validation | search_service_test.rs | 1 |
| Rebuild Operations | search_service_test.rs | 6 |
| Reset Operations | search_service_test.rs | 3 |
| Thread Safety | search_service_test.rs | 2 |
| Admin Status Response | schema_mismatch_admin_test.rs | 13 |
| Admin Status Workflow | schema_mismatch_integration_test.rs | 16 |
| **Total** | | **51** |

---

## Quick Filter by Test Type

### Unit Tests (search_service_test.rs)
- test_service_initializes_with_no_schema_mismatch_when_index_fresh
- test_search_fails_when_schema_mismatch_detected
- test_rebuild_index_clears_schema_mismatch_flag
- test_rebuild_index_clears_all_documents
- test_reset_index_behaves_like_rebuild_index
- test_can_search_and_index_after_rebuild
- test_rebuild_index_multiple_times
- test_has_schema_mismatch_is_thread_safe
- test_concurrent_rebuild_and_schema_mismatch_check
- test_schema_mismatch_flag_consistency
- test_rebuild_index_preserves_index_directory
- test_index_functionality_after_multiple_rebuilds

### API Contract Tests (schema_mismatch_admin_test.rs)
- test_search_status_response_structure_no_mismatch
- test_search_status_response_structure_with_mismatch
- test_search_status_response_json_serialization
- test_search_status_response_json_deserialization
- test_search_status_no_message_when_healthy
- test_schema_mismatch_implies_index_unavailable
- test_healthy_status_both_fields_consistent
- test_status_response_field_names_match_api_contract
- test_frontend_can_detect_schema_mismatch
- test_status_response_no_sensitive_data
- test_status_checks_consistency
- test_mismatch_message_is_actionable
- test_unavailable_index_reason_is_schema_mismatch

### Workflow/Integration Tests (schema_mismatch_integration_test.rs)
- test_admin_status_endpoint_response_format
- test_admin_status_with_mismatch_detected
- test_status_endpoint_no_auth_required
- test_admin_reset_endpoint_clears_mismatch
- test_admin_reset_response_includes_was_mismatch
- test_admin_reset_response_when_no_mismatch
- test_multiple_reset_calls_idempotent
- test_full_workflow_detect_and_reset
- test_concurrent_status_checks_during_reset
- test_status_endpoint_reflects_current_state
- test_status_consistency_in_healthy_state
- test_status_consistency_during_recovery
- test_status_not_blocked_by_admin_ops
- test_reset_error_handling
- test_status_after_rebuild
- test_mismatch_flag_independence

---

## Test Files Location

- **Unit tests**: `/workspace/klask-rs/tests/search_service_test.rs` (lines 809-1200)
- **Admin API tests**: `/workspace/klask-rs/tests/schema_mismatch_admin_test.rs`
- **Integration tests**: `/workspace/klask-rs/tests/schema_mismatch_integration_test.rs`

---

## Documentation

- **Full Documentation**: `/workspace/klask-rs/tests/SCHEMA_MISMATCH_TESTS_DOCUMENTATION.md`
- **Summary**: `/workspace/.claude/tasks/schema-mismatch-tests-summary.md`
- **Final Report**: `/workspace/SCHEMA_MISMATCH_TESTS_FINAL_REPORT.md`
- **This Index**: `/workspace/klask-rs/tests/TESTS_INDEX.md`

---

Total: **51 tests** across **3 files** with **~2,468 lines** of code and documentation.
