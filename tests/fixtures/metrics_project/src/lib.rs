pub mod domain;
mod extensions;

use domain::{Repository, Service};

pub fn bootstrap() -> Service {
    Service::new(Repository)
}
