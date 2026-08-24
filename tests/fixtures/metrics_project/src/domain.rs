use std::fmt::{self, Display};

pub trait Port {
    fn send(&self) -> usize;
    fn make() -> Self
    where
        Self: Sized;
}

pub struct Service {
    repository: Repository,
    pub(crate) requests: usize,
}

pub struct Repository;

pub enum State {
    Ready { code: u8 },
    Failed(String),
}

pub union Word {
    integer: u32,
    bytes: [u8; 4],
}

impl Service {
    pub fn new(repository: Repository) -> Self {
        Self {
            repository,
            requests: 0,
        }
    }

    pub fn execute(&self) -> usize {
        self.repository.load() + self.requests
    }

    pub fn increment(&mut self) {
        self.requests += 1;
    }
}

impl Port for Service {
    fn send(&self) -> usize {
        self.requests
    }

    fn make() -> Self {
        Self::new(Repository)
    }
}

impl Repository {
    fn load(&self) -> usize {
        1
    }
}

impl Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready { code } => write!(formatter, "ready:{code}"),
            Self::Failed(message) => formatter.write_str(message),
        }
    }
}

macro_rules! fixture_macro {
    () => {};
}

fixture_macro!();
