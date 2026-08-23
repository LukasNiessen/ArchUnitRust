//! Pure assertions for file architecture rules.

mod cycle_free;
mod cycle_violation;

pub use cycle_free::gather_cycle_violations;
pub use cycle_violation::CycleViolation;
