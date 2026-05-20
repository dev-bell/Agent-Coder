pub const ASSISTANT_RESPONSE_SCHEMA: &str = r#"{
    "type": "json_schema",
    "json_schema": {
        "name": "agent_thought_action",
        "strict": true,
        "schema": {
            "type": "object",
            "properties": {
                "Thought": { "type": "string" },
                "Action": { "type": "string" },
                "Final Answer": { "type": "string" },
            },
            "required": ["Thought"],
            "additionalProperties": false
        }
    }
}"#;