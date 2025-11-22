use std::time::Duration;
use std::thread;

pub struct DecayScheduler {
    pub tick_interval: Duration,
}

impl DecayScheduler {
    pub fn new(tick_interval: Duration) -> Self {
        Self { tick_interval }
    }

    pub fn start<F>(self, mut tick_fn: F)
    where
        F: FnMut() + Send + 'static,
    {
        thread::spawn(move || {
            loop {
                thread::sleep(self.tick_interval);
                tick_fn();
            }
        });
    }
}
