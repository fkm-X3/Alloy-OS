use alloc::collections::VecDeque;

use crate::protocol::{ClientId, DisplayEvent};

/// Minimal client contract for receiving display events.
pub trait DisplayClient {
    fn id(&self) -> ClientId;
    fn on_event(&mut self, event: &DisplayEvent);
}

/// Event queue helper for client-side buffering.
pub struct ClientEventQueue {
    client_id: ClientId,
    events: VecDeque<DisplayEvent>,
    max_events: usize,
    dropped_events: u64,
}

impl ClientEventQueue {
    pub fn new(client_id: ClientId, max_events: usize) -> Self {
        Self {
            client_id,
            events: VecDeque::with_capacity(max_events),
            max_events,
            dropped_events: 0,
        }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn enqueue(&mut self, event: DisplayEvent) {
        if self.events.len() >= self.max_events {
            self.dropped_events = self.dropped_events.saturating_add(1);
            return;
        }
        self.events.push_back(event);
    }

    pub fn dequeue(&mut self) -> Option<DisplayEvent> {
        self.events.pop_front()
    }

    pub fn pending_len(&self) -> usize {
        self.events.len()
    }

    pub fn dropped_events(&self) -> u64 {
        self.dropped_events
    }
}
