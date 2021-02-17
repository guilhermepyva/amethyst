use json::JsonValue;

#[derive(Clone, Debug)]
pub struct ChatComponent {
    json: JsonValue
}

impl ChatComponent {
    pub fn new() -> ChatComponent {
        ChatComponent {json: JsonValue::new_object()}
    }
    pub fn new_text(str: String) -> ChatComponent {
        let mut json = JsonValue::new_object();
        json["text"] = JsonValue::String(str);
        ChatComponent {json}
    }
    pub fn to_string(&self) -> String {
        self.json.to_string()
    }
    pub fn to_json(self) -> JsonValue {
        self.json
    }
}