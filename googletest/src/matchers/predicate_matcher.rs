// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(not(google3))]
use crate as googletest;
use googletest::matcher::{Matcher, MatcherResult};
use std::fmt::Debug;

/// Creates a matcher based on the predicate provided.
///
/// ```rust
/// verify_that!(3, predicate(|x: &i32| x % 2 == 1))?;  // Passes
/// ```
///
/// The predicate should take the subject type by reference and return a
/// boolean.
///
/// Note: even if the Rust compiler should be able to infer the type of
/// the closure argument, it is likely that it won't.
/// See https://github.com/rust-lang/rust/issues/12679 for update on this issue.
/// This is easily fixed by explicitly declaring the type of the argument
pub fn predicate<T: Debug + ?Sized, P>(
    predicate: P,
) -> PredicateMatcher<P, NoDescription, NoDescription>
where
    for<'a> P: Fn(&'a T) -> bool,
{
    PredicateMatcher {
        predicate,
        positive_description: NoDescription,
        negative_description: NoDescription,
    }
}

impl<P> PredicateMatcher<P, NoDescription, NoDescription> {
    /// Configures this instance to provide a more meaningful description.
    ///
    /// For example, to make sure the error message is more useful
    ///
    /// ```rust
    /// predicate(|x: &i32| x % 2 == 1)
    ///   .with_description("is odd", "is even")
    /// ```
    ///
    /// This is optional as it only provides value when the test fails.
    ///
    /// Description can be passed by &str, String of Fn() -> Into<String>
    pub fn with_description<D1: PredicateDescription, D2: PredicateDescription>(
        self,
        positive_description: D1,
        negative_description: D2,
    ) -> PredicateMatcher<P, D1, D2> {
        PredicateMatcher { predicate: self.predicate, positive_description, negative_description }
    }
}

/// A matcher which applies `predicate` on the value.
///
/// See [`predicate`].
pub struct PredicateMatcher<P, D1, D2> {
    predicate: P,
    positive_description: D1,
    negative_description: D2,
}

/// A trait to allow [`with_description`] to accept multiple types.
///
/// See [`with_description`]
pub trait PredicateDescription {
    fn to_description(&self) -> String;
}

impl PredicateDescription for &str {
    fn to_description(&self) -> String {
        self.to_string()
    }
}

impl PredicateDescription for String {
    fn to_description(&self) -> String {
        self.clone()
    }
}

impl<T, S> PredicateDescription for T
where
    T: Fn() -> S,
    S: Into<String>,
{
    fn to_description(&self) -> String {
        self().into()
    }
}

// Sentinel type to tag a MatcherBuilder as without a description.
#[doc(hidden)]
pub struct NoDescription;

impl<T: Debug, P> Matcher<T> for PredicateMatcher<P, NoDescription, NoDescription>
where
    for<'a> P: Fn(&'a T) -> bool,
{
    fn matches(&self, actual: &T) -> MatcherResult {
        if (self.predicate)(actual) { MatcherResult::Matches } else { MatcherResult::DoesNotMatch }
    }

    fn describe(&self, result: MatcherResult) -> String {
        match result {
            MatcherResult::Matches => "matches".to_string(),
            MatcherResult::DoesNotMatch => "does not match".to_string(),
        }
    }
}

impl<T: Debug, P, D1: PredicateDescription, D2: PredicateDescription> Matcher<T>
    for PredicateMatcher<P, D1, D2>
where
    for<'a> P: Fn(&'a T) -> bool,
{
    fn matches(&self, actual: &T) -> MatcherResult {
        if (self.predicate)(actual) { MatcherResult::Matches } else { MatcherResult::DoesNotMatch }
    }

    fn describe(&self, result: MatcherResult) -> String {
        match result {
            MatcherResult::Matches => self.positive_description.to_description(),
            MatcherResult::DoesNotMatch => self.negative_description.to_description(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(google3))]
    use crate as googletest;
    #[cfg(not(google3))]
    use googletest::matchers;
    use googletest::{google_test, verify_that, Result};
    use matchers::{displays_as, eq};

    // Simple matcher with a description
    fn is_odd() -> impl Matcher<i32> {
        predicate(|x| x % 2 == 1).with_description("is odd", "is even")
    }

    #[google_test]
    fn predicate_matcher_odd() -> Result<()> {
        verify_that!(1, is_odd())
    }

    #[google_test]
    fn predicate_matcher_odd_explain_match_matches() -> Result<()> {
        verify_that!(is_odd().explain_match(&1), displays_as(eq("which is odd")))
    }

    #[google_test]
    fn predicate_matcher_odd_explain_match_does_not_match() -> Result<()> {
        verify_that!(is_odd().explain_match(&2), displays_as(eq("which is even")))
    }

    // Simple Matcher without description
    fn is_even() -> impl Matcher<i32> {
        predicate(|x| x % 2 == 0)
    }

    #[google_test]
    fn predicate_matcher_even() -> Result<()> {
        verify_that!(2, is_even())
    }

    #[google_test]
    fn predicate_matcher_even_explain_match_matches() -> Result<()> {
        verify_that!(is_even().explain_match(&2), displays_as(eq("which matches")))
    }

    #[google_test]
    fn predicate_matcher_even_explain_match_does_not_match() -> Result<()> {
        verify_that!(is_even().explain_match(&1), displays_as(eq("which does not match")))
    }

    #[google_test]
    fn predicate_matcher_generator_lambda() -> Result<()> {
        let is_divisible_by = |quotient| {
            predicate(move |x: &i32| x % quotient == 0).with_description(
                move || format!("is divisible by {quotient}"),
                move || format!("is not divisible by {quotient}"),
            )
        };
        verify_that!(49, is_divisible_by(7))
    }

    #[google_test]
    fn predicate_matcher_inline() -> Result<()> {
        verify_that!(2048, predicate(|x: &i32| x.count_ones() == 1))
    }

    #[google_test]
    fn predicate_matcher_function_pointer() -> Result<()> {
        use std::time::Duration;
        verify_that!(Duration::new(0, 0), predicate(Duration::is_zero))
    }
}