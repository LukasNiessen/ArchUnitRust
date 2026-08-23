//! The execution contract shared by every terminal architecture rule.

use crate::common::error::ArchUnitError;
use crate::common::fluentapi::CheckOptions;
use crate::violation::Violation;

/// The complete outcome of running one architecture rule.
///
/// `Ok(Vec::new())` passes, `Ok` with violations is an architecture disagreement, and `Err` means
/// no verdict could be reached.
pub type CheckResult = Result<Vec<Violation>, ArchUnitError>;

/// A terminal architecture rule that can judge a project.
///
/// Building a fluent rule is lazy; only these methods may touch the filesystem. Test helpers and
/// report consumers depend on this trait rather than on individual rule families.
pub trait Checkable {
    /// Runs the rule with the strict, quiet defaults from [`CheckOptions`].
    fn check(&self) -> CheckResult {
        self.check_with(&CheckOptions::default())
    }

    /// Runs the rule with an explicit immutable options bag.
    fn check_with(&self, options: &CheckOptions) -> CheckResult;
}

#[cfg(test)]
mod tests {
    use super::{CheckResult, Checkable};
    use crate::{ArchUnitError, CheckOptions, TechnicalError};

    struct OptionEchoRule;

    impl Checkable for OptionEchoRule {
        fn check_with(&self, options: &CheckOptions) -> CheckResult {
            if options.clears_cache() {
                Err(ArchUnitError::from(TechnicalError::new(
                    "fixture cache clear failed",
                )))
            } else {
                Ok(Vec::new())
            }
        }
    }

    fn run_default(rule: &dyn Checkable) -> CheckResult {
        rule.check()
    }

    #[test]
    fn default_check_uses_defaults_through_an_object_safe_contract() {
        let result = run_default(&OptionEchoRule);

        assert!(matches!(result, Ok(violations) if violations.is_empty()));
    }

    #[test]
    fn explicit_options_reach_the_terminal_by_shared_reference() {
        let options = CheckOptions::new().with_clear_cache(true);
        let result = OptionEchoRule.check_with(&options);

        assert!(matches!(result, Err(ArchUnitError::Technical(_))));
        assert!(options.clears_cache());
    }
}
