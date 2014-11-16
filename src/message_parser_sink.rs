use events::{MessageParserEvent, MessageParserStage};

pub struct MessageParserSink {
    events: Vec<MessageParserEvent>
}

impl MessageParserSink {
    pub fn new() -> MessageParserSink {
        MessageParserSink{ events: vec![] }
    }

    pub fn events(&self) -> Vec<MessageParserEvent> {
        self.events.clone()
    }
}

impl MessageParserStage for MessageParserSink {
    fn process_event(&mut self, event: MessageParserEvent) {
        self.events.push(event);
    }
}
