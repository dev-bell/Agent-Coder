pub const ASSISTANT_RESPONSE_SCHEMA: &str = r#"{
    "name": "agent_thought_action",
    "strict": true,
    "schema": {
        "type": "object",
        "properties": {
            "Thought": { "type": "string" },
            "Action": { "type": ["string", "null"] },
            "Final Answer": { "type": ["string", "null"] }
        },
        "required": ["Thought", "Action", "Final Answer"],
        "additionalProperties": false
    }
}"#;