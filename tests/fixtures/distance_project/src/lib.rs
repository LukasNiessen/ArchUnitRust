pub mod first;
pub mod second;

pub trait Gateway {
    fn execute(&self);
}
