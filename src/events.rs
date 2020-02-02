use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyEvent};

#[derive(Clone)]
pub struct KeyEventQueue<T: Send + Copy> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send + Copy> KeyEventQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn get_latest_event(&self) -> Option<T> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            let el = queue.pop_back();
            queue.clear();
            return el;
        } else {
            panic!("poisoned mutex");
        }
    }

    pub fn get_all_events(&self) -> Option<Vec<T>> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            let drained = queue.drain(..).collect::<Vec<_>>();
            queue.clear();
            return Some(drained);
        } else {
            panic!("poisoned mutex");
        }
    }

    fn add_event(&self, event: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(event);
            queue.len()
        } else {
            panic!("poisoned mutex");
        }
    }
}

pub fn send_events(event_queue: KeyEventQueue<KeyEvent>) -> crossterm::Result<()> {
    loop {
        if poll(Duration::from_millis(3))? {
            match read()? {
                // will not block
                Event::Key(event) => {
                    event_queue.add_event(event);
                }
                Event::Mouse(_event) => {}
                Event::Resize(_width, _height) => {}
            }
        }
    }
}
