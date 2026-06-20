pub const ASSISTANT_RESPONSE_SCHEMA: &str = r#"{
    "name": "agent_thought_action",
    "strict": true,
    "schema": {
        "type": "object",
        "properties": {
            "Thought": { "type": "string" },
            "Action": { "anyOf":{{"type": "string"}, {"type":"null"}} },
            "Final Answer": { "anyOf":{{"type": "string"}, {"type:"null"}} }
        },
        "required": ["Thought"],
        "additionalProperties": false
    }
}"#;