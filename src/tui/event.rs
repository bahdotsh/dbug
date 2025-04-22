use crate::prelude::*;
use crossterm::event::{self, Event};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Terminal event handler
pub struct Events {
    rx: mpsc::Receiver<Event>,
    _tx: mpsc::Sender<Event>,
    tick_rate: Duration,
    last_tick: Instant,
}

impl Events {
    /// Create a new event handler with the given tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let _tx = tx.clone();

        // Spawn a thread to handle events
        thread::spawn(move || {
            loop {
                // Poll for events, with a timeout
                if event::poll(Duration::from_millis(10)).unwrap() {
                    if let Ok(event) = event::read() {
                        // If sending fails, the receiver has been dropped
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                }

                // Sleep a bit to avoid consuming too much CPU
                thread::sleep(Duration::from_millis(10));
            }
        });

        Self {
            rx,
            _tx,
            tick_rate,
            last_tick: Instant::now(),
        }
    }

    /// Get the next event, which can be either a terminal event or a tick
    pub fn next(&mut self) -> DbugResult<Option<Event>> {
        // Check if we've received an event
        if let Ok(event) = self.rx.try_recv() {
            return Ok(Some(event));
        }

        // If not, check if we should tick
        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            // Return None for a tick, which allows the app to update its state
            return Ok(None);
        }

        // No event and no tick
        Ok(None)
    }
}
