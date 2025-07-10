use std::num::NonZeroUsize;
use std::{cmp::Ordering, sync::Mutex};

use lru::LruCache;
use regex::Regex;

use super::Matcher;
use crate::{Error, Result};

pub struct Regexp {
    lru: Mutex<LruCache<String, Regex>>,
}

impl Regexp {
    pub fn new(cache_size: usize) -> Result<Self> {
        Ok(Self {
            lru: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(cache_size).ok_or(Error::InvalidCacheSize(cache_size))?,
            )),
        })
    }
}

impl Matcher for Regexp {
    fn matches(
        &self,
        delimiter_start: char,
        delimiter_end: char,
        haystack: Vec<String>,
        needle: &str,
    ) -> Result<bool> {
        for h in haystack.iter() {
            if !h.contains(delimiter_start) {
                if h.eq(needle) {
                    return Ok(true);
                }
                continue;
            }
            {
                let mut rlru = self
                    .lru
                    .lock()
                    .map_err(|err| Error::LockError(format!("{err}")))?;
                if let Some(reg) = rlru.get(h) {
                    if reg.is_match(needle) {
                        return Ok(true);
                    }
                    continue;
                }
            };

            let pattern = build_regex(h, delimiter_start, delimiter_end)?;
            let reg = Regex::new(pattern.as_str()).map_err(Error::CompileRegexError)?;
            {
                let mut wlru = self
                    .lru
                    .lock()
                    .map_err(|err| Error::LockError(format!("{err}")))?;
                wlru.put(h.to_owned(), reg.clone());
            };

            if reg.is_match(needle) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn delimiter_indices(s: &str, delimiter_start: char, delimiter_end: char) -> Result<Vec<usize>> {
    let (mut level, mut idx) = (0, 0);
    let mut idxs: Vec<usize> = Vec::new();
    for (i, value) in s.chars().enumerate() {
        if value == delimiter_start {
            level += 1;
            if level == 1 {
                idx = i;
            }
        } else if value == delimiter_end {
            level -= 1;
            match level.cmp(&0) {
                Ordering::Less => {
                    return Err(Error::UnbalancedBraces(s.to_owned()));
                }
                Ordering::Equal => {
                    idxs.push(idx);
                    idxs.push(i + 1);
                }
                Ordering::Greater => {}
            }
        }
    }
    if level != 0 {
        return Err(Error::UnbalancedBraces(s.to_owned()));
    }
    Ok(idxs)
}

fn build_regex(tpl: &str, delimiter_start: char, delimiter_end: char) -> Result<String> {
    let idx = delimiter_indices(tpl, delimiter_start, delimiter_end)?;
    let mut buffer = String::new();
    buffer.push('^');
    let (mut i, mut end) = (0, 0);
    loop {
        if i >= idx.len() {
            break;
        }
        let temp_id = match idx.get(i) {
            Some(v) => v.to_owned(),
            None => return Err(Error::NotIndex(format!("not index {i} in {idx:?}"))),
        };
        let raw = match tpl.get(end..temp_id) {
            Some(v) => v,
            None => {
                return Err(Error::NotIndex(format!(
                    "not index {end} to {temp_id} in {tpl:?}"
                )));
            }
        };

        end = match idx.get(i + 1) {
            Some(v) => v.to_owned(),
            None => return Err(Error::NotIndex(format!("not index {} in {:?}", i + 1, idx))),
        };
        let patt = match tpl.get(temp_id + 1..end - 1) {
            Some(v) => v,
            None => {
                return Err(Error::NotIndex(format!(
                    "not index {} to {} in {:?}",
                    temp_id + 1,
                    end - 1,
                    tpl
                )))
            }
        };
        buffer.push_str(format!("{}({})", regex::escape(raw), patt).as_str());
        Regex::new(format!("^{patt}$").as_str()).map_err(Error::CompileRegexError)?;
        i += 2;
    }
    let raw = match tpl.get(end..) {
        Some(v) => v,
        None => {
            return Err(Error::NotIndex(format!(
                "not index {end} to end in {tpl:?}"
            )))
        }
    };
    buffer.push_str(regex::escape(raw).as_str());
    buffer.push('$');
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg() {
        let reg = regex::Regex::new("^(reate|delete)$").unwrap();
        assert!(reg.is_match("delete"))
    }
    #[test]
    fn build() {
        assert_eq!(
            build_regex("<create|delete>", '<', '>').unwrap(),
            "^(create|delete)$".to_owned()
        )
    }
}
