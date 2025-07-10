pub(crate) mod reg;

use crate::Result;

pub trait Matcher {
    fn matches(
        &self,
        delimiter_start: char,
        delimiter_end: char,
        haystack: Vec<String>,
        needle: &str,
    ) -> Result<bool>;
}
