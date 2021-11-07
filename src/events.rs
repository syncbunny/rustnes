enum EventType {
	NMI
}

pub struct Event {
	event_type: u32,
}

pub struct EventQueue {
}

impl EventQueue {
	pub fn new() -> EventQueue {
		EventQueue {
		}
	}
}


