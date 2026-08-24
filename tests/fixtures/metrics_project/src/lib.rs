pub mod domain;

use domain::{Repository, Service};

pub fn bootstrap() -> Service {
    Service::new(Repository)
}
