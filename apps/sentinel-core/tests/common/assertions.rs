use serde_json::Value;

#[allow(dead_code)]
pub fn assert_api_envelope_shape(v: &Value) {
    let obj = v.as_object().expect("response body must be a JSON object");

    for key in ["success", "data", "error", "timestamp", "request_id"] {
        assert!(
            obj.contains_key(key),
            "missing top-level key '{key}' in response: {v}"
        );
    }

    assert!(obj["success"].is_boolean(), "'success' must be boolean");
    assert!(obj["timestamp"].is_string(), "'timestamp' must be string");
    assert!(obj["request_id"].is_string(), "'request_id' must be string");

    // error: null OR object with known fields
    if obj["error"].is_null() {
        // ok
    } else {
        let err = obj["error"]
            .as_object()
            .expect("'error' must be null or an object");

        for key in ["code", "message"] {
            assert!(
                err.contains_key(key),
                "missing error key '{key}' in response: {v}"
            );
        }

        assert!(err["code"].is_string(), "'error.code' must be string");
        assert!(
            err["message"].is_string() || err["message"].is_null(),
            "'error.message' must be string or null"
        );
        /*
        assert!(
            err["details"].is_object() || err["details"].is_null(),
            "'error.details' must be object or null"
        );
        assert!(err["status"].is_number(), "'error.status' must be number");
        */
    }

    // Guardrail: no unexpected top-level fields
    let allowed = ["success", "data", "error", "timestamp", "request_id"];
    for key in obj.keys() {
        assert!(
            allowed.contains(&key.as_str()),
            "unexpected top-level key '{key}' in response: {v}"
        );
    }
}
