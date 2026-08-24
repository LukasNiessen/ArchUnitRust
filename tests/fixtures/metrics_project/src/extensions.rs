use crate::domain::Service;

impl Service {
    pub fn reset(&mut self) {
        self.requests = 0;
    }
}
