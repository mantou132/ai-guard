use bytes::Bytes;
use serde_json::{Map, Value, json};

use crate::models::ApiType;

const MAX_RECENT_MESSAGES: usize = 16;
const MAX_TOOLS: usize = 24;
const MAX_CONTENT_PARTS: usize = 32;
const MAX_ARRAY_ITEMS: usize = 32;
const MAX_OBJECT_FIELDS: usize = 48;

pub fn extract_request_payload(api_type: &ApiType, bytes: &Bytes, limit: usize) -> Value {
    if bytes.is_empty() {
        return Value::Null;
    }

    match serde_json::from_slice::<Value>(bytes) {
        Ok(value) => request_payload_from_value(api_type, &value, limit),
        Err(_) => json!({
            "body": truncate(&String::from_utf8_lossy(bytes), value_limit(limit)),
        }),
    }
}

pub fn extract_response_payload(api_type: &ApiType, bytes: &Bytes, limit: usize) -> Value {
    if bytes.is_empty() {
        return Value::Null;
    }

    match serde_json::from_slice::<Value>(bytes) {
        Ok(value) => response_payload_from_value(api_type, &value, limit),
        Err(_) => {
            let text = String::from_utf8_lossy(bytes);
            if text
                .lines()
                .any(|line| line.trim_start().starts_with("data:"))
            {
                extract_sse_payload(&text, limit)
            } else {
                json!({
                    "body": truncate(&text, value_limit(limit)),
                })
            }
        }
    }
}

pub fn extract_model(bytes: &Bytes) -> Option<String> {
    let value = serde_json::from_slice::<Value>(bytes).ok()?;
    value
        .get("model")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn request_payload_from_value(api_type: &ApiType, value: &Value, limit: usize) -> Value {
    let mut output = Map::new();
    output.insert(
        "api_type".to_string(),
        Value::String(api_type_name(api_type).to_string()),
    );

    insert_string(&mut output, "model", value.get("model"), limit);
    insert_string(
        &mut output,
        "conversation_id",
        value.get("conversation_id"),
        limit,
    );
    insert_string(&mut output, "thread_id", value.get("thread_id"), limit);
    insert_metadata_hint(&mut output, value.get("metadata"), limit);

    if let Some(system) = value.get("system") {
        output.insert("system".to_string(), compact_content(system, limit));
    }
    if let Some(messages) = value.get("messages") {
        output.insert("messages".to_string(), extract_messages(messages, limit));
    }
    if let Some(input) = value.get("input") {
        output.insert("input".to_string(), compact_content(input, limit));
    }
    if let Some(instructions) = value.get("instructions") {
        output.insert(
            "instructions".to_string(),
            compact_content(instructions, limit),
        );
    }
    if let Some(tools) = value.get("tools") {
        output.insert("tools".to_string(), extract_tools(tools, limit));
    }
    if let Some(tool_choice) = value.get("tool_choice") {
        output.insert("tool_choice".to_string(), compact_value(tool_choice, limit));
    }
    if let Some(response_format) = value.get("response_format") {
        output.insert(
            "response_format".to_string(),
            compact_value(response_format, limit),
        );
    }

    Value::Object(output)
}

fn response_payload_from_value(api_type: &ApiType, value: &Value, limit: usize) -> Value {
    let mut output = Map::new();
    output.insert(
        "api_type".to_string(),
        Value::String(api_type_name(api_type).to_string()),
    );

    insert_string(&mut output, "model", value.get("model"), limit);
    if let Some(error) = value.get("error") {
        output.insert("error".to_string(), compact_value(error, limit));
    }
    if let Some(choices) = value.get("choices") {
        output.insert("choices".to_string(), extract_choices(choices, limit));
    }
    if let Some(content) = value.get("content") {
        output.insert("content".to_string(), compact_content(content, limit));
    }
    if let Some(output_value) = value.get("output") {
        output.insert("output".to_string(), compact_content(output_value, limit));
    }
    if let Some(stop_reason) = value.get("stop_reason") {
        output.insert("stop_reason".to_string(), compact_value(stop_reason, limit));
    }

    Value::Object(output)
}

fn extract_messages(value: &Value, limit: usize) -> Value {
    let Some(messages) = value.as_array() else {
        return compact_content(value, limit);
    };

    let recent_start = messages.len().saturating_sub(MAX_RECENT_MESSAGES);
    let mut omitted = 0usize;
    let items = messages
        .iter()
        .enumerate()
        .filter_map(|(index, message)| {
            let role = message.get("role").and_then(Value::as_str);
            let keep = matches!(role, Some("system" | "developer")) || index >= recent_start;
            if keep {
                Some(extract_message(message, limit))
            } else {
                omitted += 1;
                None
            }
        })
        .collect::<Vec<_>>();

    json!({
        "total": messages.len(),
        "omitted_older": omitted,
        "items": items,
    })
}

fn extract_message(value: &Value, limit: usize) -> Value {
    let Some(object) = value.as_object() else {
        return compact_content(value, limit);
    };

    let mut output = Map::new();
    insert_string(&mut output, "role", object.get("role"), limit);
    insert_string(&mut output, "name", object.get("name"), limit);
    insert_string(
        &mut output,
        "tool_call_id",
        object.get("tool_call_id"),
        limit,
    );

    if let Some(content) = object.get("content") {
        output.insert("content".to_string(), compact_content(content, limit));
    }
    if let Some(tool_calls) = object.get("tool_calls") {
        output.insert(
            "tool_calls".to_string(),
            extract_tool_calls(tool_calls, limit),
        );
    }
    if let Some(function_call) = object.get("function_call") {
        output.insert(
            "function_call".to_string(),
            compact_value(function_call, limit),
        );
    }

    Value::Object(output)
}

fn extract_choices(value: &Value, limit: usize) -> Value {
    let Some(choices) = value.as_array() else {
        return compact_value(value, limit);
    };

    Value::Array(
        choices
            .iter()
            .take(MAX_ARRAY_ITEMS)
            .map(|choice| {
                let Some(object) = choice.as_object() else {
                    return compact_value(choice, limit);
                };

                let mut output = Map::new();
                insert_compact(&mut output, "index", object.get("index"), limit);
                insert_string(
                    &mut output,
                    "finish_reason",
                    object.get("finish_reason"),
                    limit,
                );
                if let Some(message) = object.get("message") {
                    output.insert("message".to_string(), extract_message(message, limit));
                }
                if let Some(delta) = object.get("delta") {
                    output.insert("delta".to_string(), extract_message(delta, limit));
                }
                Value::Object(output)
            })
            .collect(),
    )
}

fn extract_tools(value: &Value, limit: usize) -> Value {
    let Some(tools) = value.as_array() else {
        return compact_value(value, limit);
    };

    Value::Array(
        tools
            .iter()
            .take(MAX_TOOLS)
            .map(|tool| extract_tool(tool, limit))
            .collect(),
    )
}

fn extract_tool(value: &Value, limit: usize) -> Value {
    let Some(object) = value.as_object() else {
        return compact_value(value, limit);
    };

    let mut output = Map::new();
    insert_string(&mut output, "type", object.get("type"), limit);
    insert_string(&mut output, "name", object.get("name"), limit);
    insert_string(&mut output, "description", object.get("description"), limit);

    if let Some(function) = object.get("function") {
        output.insert("function".to_string(), compact_function(function, limit));
    }
    if let Some(input_schema) = object.get("input_schema") {
        output.insert(
            "input_schema".to_string(),
            compact_value(input_schema, limit),
        );
    }
    if let Some(parameters) = object.get("parameters") {
        output.insert("parameters".to_string(), compact_value(parameters, limit));
    }

    Value::Object(output)
}

fn compact_function(value: &Value, limit: usize) -> Value {
    let Some(object) = value.as_object() else {
        return compact_value(value, limit);
    };

    let mut output = Map::new();
    insert_string(&mut output, "name", object.get("name"), limit);
    insert_string(&mut output, "description", object.get("description"), limit);
    if let Some(parameters) = object.get("parameters") {
        output.insert("parameters".to_string(), compact_value(parameters, limit));
    }
    if let Some(arguments) = object.get("arguments") {
        output.insert("arguments".to_string(), compact_value(arguments, limit));
    }
    Value::Object(output)
}

fn extract_tool_calls(value: &Value, limit: usize) -> Value {
    let Some(tool_calls) = value.as_array() else {
        return compact_value(value, limit);
    };

    Value::Array(
        tool_calls
            .iter()
            .take(MAX_ARRAY_ITEMS)
            .map(|tool_call| {
                let Some(object) = tool_call.as_object() else {
                    return compact_value(tool_call, limit);
                };

                let mut output = Map::new();
                insert_string(&mut output, "id", object.get("id"), limit);
                insert_string(&mut output, "type", object.get("type"), limit);
                insert_string(&mut output, "name", object.get("name"), limit);
                if let Some(function) = object.get("function") {
                    output.insert("function".to_string(), compact_function(function, limit));
                }
                if let Some(input) = object.get("input") {
                    output.insert("input".to_string(), compact_value(input, limit));
                }
                Value::Object(output)
            })
            .collect(),
    )
}

fn compact_content(value: &Value, limit: usize) -> Value {
    match value {
        Value::String(text) => Value::String(truncate(text, value_limit(limit))),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .take(MAX_CONTENT_PARTS)
                .map(|item| compact_content_part(item, limit))
                .collect(),
        ),
        _ => compact_value(value, limit),
    }
}

fn compact_content_part(value: &Value, limit: usize) -> Value {
    let Some(object) = value.as_object() else {
        return compact_value(value, limit);
    };

    let mut output = Map::new();
    insert_string(&mut output, "type", object.get("type"), limit);
    insert_string(&mut output, "id", object.get("id"), limit);
    insert_string(&mut output, "name", object.get("name"), limit);
    insert_string(&mut output, "tool_use_id", object.get("tool_use_id"), limit);
    insert_string(
        &mut output,
        "tool_call_id",
        object.get("tool_call_id"),
        limit,
    );

    for key in ["text", "content", "input", "arguments", "partial_json"] {
        if let Some(value) = object.get(key) {
            output.insert(key.to_string(), compact_value(value, limit));
        }
    }

    for key in ["image_url", "source", "file", "audio"] {
        if object.contains_key(key) {
            output.insert(
                key.to_string(),
                Value::String("[media omitted]".to_string()),
            );
        }
    }

    Value::Object(output)
}

fn extract_sse_payload(text: &str, limit: usize) -> Value {
    let mut content = String::new();
    let mut events_captured = 0usize;
    let mut done_seen = false;
    let mut tool_events = Vec::new();

    for line in text.lines() {
        let line = line.trim_start();
        if !line.starts_with("data:") {
            continue;
        }

        let data = line.trim_start_matches("data:").trim();
        if data == "[DONE]" {
            done_seen = true;
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        events_captured += 1;

        append_stream_text(&mut content, &value, limit);
        collect_stream_tool_events(&mut tool_events, &value, limit);
    }

    let mut output = Map::new();
    output.insert("stream".to_string(), Value::Bool(true));
    output.insert("events_captured".to_string(), json!(events_captured));
    output.insert("done_seen".to_string(), Value::Bool(done_seen));
    if !content.is_empty() {
        output.insert(
            "content".to_string(),
            Value::String(truncate(&content, value_limit(limit))),
        );
    }
    if !tool_events.is_empty() {
        output.insert("tool_events".to_string(), Value::Array(tool_events));
    }

    Value::Object(output)
}

fn append_stream_text(content: &mut String, value: &Value, limit: usize) {
    append_string_from_pointer(content, value, "/delta/text", limit);
    append_string_from_pointer(content, value, "/delta", limit);

    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            append_string_from_pointer(content, choice, "/delta/content", limit);
            append_string_from_pointer(content, choice, "/message/content", limit);
        }
    }
}

fn append_string_from_pointer(content: &mut String, value: &Value, pointer: &str, limit: usize) {
    let Some(text) = value.pointer(pointer).and_then(Value::as_str) else {
        return;
    };
    if content.len() >= value_limit(limit) {
        return;
    }
    content.push_str(text);
}

fn collect_stream_tool_events(events: &mut Vec<Value>, value: &Value, limit: usize) {
    if events.len() >= MAX_ARRAY_ITEMS {
        return;
    }

    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            if let Some(tool_calls) = choice.pointer("/delta/tool_calls") {
                events.push(extract_tool_calls(tool_calls, limit));
            }
        }
    }

    if let Some(content_block) = value.get("content_block") {
        let block_type = content_block.get("type").and_then(Value::as_str);
        if matches!(block_type, Some("tool_use")) {
            events.push(compact_content_part(content_block, limit));
        }
    }

    if let Some(partial_json) = value.pointer("/delta/partial_json").and_then(Value::as_str) {
        events.push(json!({
            "partial_json": truncate(partial_json, value_limit(limit)),
        }));
    }
}

fn compact_value(value: &Value, limit: usize) -> Value {
    match value {
        Value::String(text) => Value::String(truncate(text, value_limit(limit))),
        Value::Array(items) => {
            let mut output = items
                .iter()
                .take(MAX_ARRAY_ITEMS)
                .map(|item| compact_value(item, limit))
                .collect::<Vec<_>>();
            if items.len() > MAX_ARRAY_ITEMS {
                output.push(json!({ "omitted_items": items.len() - MAX_ARRAY_ITEMS }));
            }
            Value::Array(output)
        }
        Value::Object(object) => Value::Object(
            object
                .iter()
                .take(MAX_OBJECT_FIELDS)
                .map(|(key, value)| (key.clone(), compact_value(value, limit)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn insert_string(output: &mut Map<String, Value>, key: &str, value: Option<&Value>, limit: usize) {
    let Some(text) = value.and_then(Value::as_str) else {
        return;
    };
    output.insert(
        key.to_string(),
        Value::String(truncate(text, value_limit(limit))),
    );
}

fn insert_compact(output: &mut Map<String, Value>, key: &str, value: Option<&Value>, limit: usize) {
    let Some(value) = value else {
        return;
    };
    output.insert(key.to_string(), compact_value(value, limit));
}

fn insert_metadata_hint(output: &mut Map<String, Value>, metadata: Option<&Value>, limit: usize) {
    let Some(metadata) = metadata.and_then(Value::as_object) else {
        return;
    };

    let mut hints = Map::new();
    for key in ["conversation_id", "session_id", "thread_id", "user"] {
        insert_string(&mut hints, key, metadata.get(key), limit);
    }
    if !hints.is_empty() {
        output.insert("metadata".to_string(), Value::Object(hints));
    }
}

fn api_type_name(api_type: &ApiType) -> &'static str {
    match api_type {
        ApiType::OpenAi => "open_ai",
        ApiType::Anthropic => "anthropic",
    }
}

fn value_limit(limit: usize) -> usize {
    if limit == 0 {
        0
    } else {
        limit.clamp(512, 4096)
    }
}

pub fn truncate(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        return value.to_string();
    }

    let mut output = value
        .chars()
        .scan(0usize, |size, ch| {
            let next = *size + ch.len_utf8();
            if next > limit {
                None
            } else {
                *size = next;
                Some(ch)
            }
        })
        .collect::<String>();
    output.push_str("\n...[truncated]");
    output
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn extracts_openai_chat_response() {
        let body = Bytes::from_static(
            br#"{
                "id": "chatcmpl_1",
                "object": "chat.completion",
                "model": "gpt-4o",
                "choices": [
                    {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "I will call a tool.",
                            "tool_calls": [
                                {
                                    "id": "call_1",
                                    "type": "function",
                                    "function": {
                                        "name": "read_file",
                                        "arguments": "{\"path\":\"src/main.rs\"}"
                                    }
                                }
                            ]
                        },
                        "finish_reason": "tool_calls"
                    }
                ],
                "usage": { "total_tokens": 100 }
            }"#,
        );

        let payload = extract_response_payload(&ApiType::OpenAi, &body, 4096);

        assert_eq!(
            payload.pointer("/api_type").and_then(Value::as_str),
            Some("open_ai")
        );
        assert_eq!(
            payload
                .pointer("/choices/0/message/content")
                .and_then(Value::as_str),
            Some("I will call a tool.")
        );
        assert_eq!(
            payload
                .pointer("/choices/0/message/tool_calls/0/function/name")
                .and_then(Value::as_str),
            Some("read_file")
        );
        assert!(payload.pointer("/usage").is_none());
    }

    #[test]
    fn extracts_anthropic_message_response() {
        let body = Bytes::from_static(
            br#"{
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "model": "claude-sonnet-4",
                "content": [
                    { "type": "text", "text": "I will search." },
                    {
                        "type": "tool_use",
                        "id": "toolu_1",
                        "name": "web_search",
                        "input": { "query": "security review" }
                    }
                ],
                "stop_reason": "tool_use",
                "usage": { "input_tokens": 10, "output_tokens": 20 }
            }"#,
        );

        let payload = extract_response_payload(&ApiType::Anthropic, &body, 4096);

        assert_eq!(
            payload.pointer("/api_type").and_then(Value::as_str),
            Some("anthropic")
        );
        assert_eq!(
            payload.pointer("/content/0/text").and_then(Value::as_str),
            Some("I will search.")
        );
        assert_eq!(
            payload.pointer("/content/1/name").and_then(Value::as_str),
            Some("web_search")
        );
        assert_eq!(
            payload
                .pointer("/content/1/input/query")
                .and_then(Value::as_str),
            Some("security review")
        );
        assert_eq!(
            payload.pointer("/stop_reason").and_then(Value::as_str),
            Some("tool_use")
        );
        assert!(payload.pointer("/usage").is_none());
    }

    #[test]
    fn extracts_openai_stream_response() {
        let body = Bytes::from_static(
            br#"data: {"choices":[{"delta":{"role":"assistant","content":"Hel"}}]}
data: {"choices":[{"delta":{"content":"lo"}}]}
data: [DONE]
"#,
        );

        let payload = extract_response_payload(&ApiType::OpenAi, &body, 4096);

        assert_eq!(
            payload.pointer("/stream").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            payload.pointer("/content").and_then(Value::as_str),
            Some("Hello")
        );
        assert_eq!(
            payload.pointer("/done_seen").and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn extracts_anthropic_stream_response() {
        let body = Bytes::from_static(
            br#"event: content_block_delta
data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"Hel"}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"lo"}}

event: content_block_start
data: {"type":"content_block_start","content_block":{"type":"tool_use","id":"toolu_1","name":"lookup","input":{}}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"{\"q\":\"risk\"}"}}
"#,
        );

        let payload = extract_response_payload(&ApiType::Anthropic, &body, 4096);

        assert_eq!(
            payload.pointer("/stream").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            payload.pointer("/content").and_then(Value::as_str),
            Some("Hello")
        );
        assert_eq!(
            payload
                .pointer("/tool_events/0/name")
                .and_then(Value::as_str),
            Some("lookup")
        );
        assert_eq!(
            payload
                .pointer("/tool_events/1/partial_json")
                .and_then(Value::as_str),
            Some("{\"q\":\"risk\"}")
        );
    }
}
