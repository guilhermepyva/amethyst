use json::JsonValue;

#[derive(Debug)]
pub struct ChatComponent {
    json: JsonValue
}

impl ChatComponent {
    pub fn new() -> ChatComponent {
        ChatComponent {json: JsonValue::new_object()}
    }
    pub fn new_text(str: String) -> ChatComponent {
        let mut json = JsonValue::new_object();
        json.insert("text", JsonValue::String(str)).unwrap();
        ChatComponent {json}
    }
    pub fn to_json(&self) -> String {
        self.json.to_string()
    }
}