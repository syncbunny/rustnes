use std::collections::VecDeque;

pub enum EventType {
	NMI
}

pub struct Event {
	pub event_type: EventType,
}

pub struct EventQueue {
	queue: VecDeque<Event>
}

impl EventQueue {
	pub fn new() -> EventQueue {
		EventQueue {
			queue: VecDeque::new()
		}
	}

	pub fn push(&mut self, event: Event) {
		self.queue.push_back(event);
	}

	pub fn pop(&mut self) -> Option<Event> {
		self.queue.pop_front()
	}
}

impl Event {
	pub fn new(event_type: EventType) -> Event {
		Event {
			event_type: event_type
		}
	}
}

