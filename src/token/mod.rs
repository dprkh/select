// MIT License
//
// Copyright (c) 2025 Dmytro Prokhorov
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::fmt;

/// A newtype for representing an estimated token count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TokenCount(pub usize);

impl fmt::Display for TokenCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Estimates the number of tokens in a given text using a hybrid heuristic.
///
/// This is a rough approximation that provides a more balanced estimate for both
/// prose and code compared to simple character counting. It's based on the following:
/// - Character-based estimate: Assumes an average of 4 characters per token.
///   This tends to be a reasonable upper bound for code.
/// - Word-based estimate: Assumes 1 token is roughly 0.75 words (or 4 tokens per 3 words).
///   This is often more accurate for natural language text.
///
/// The function returns the maximum of these two estimates to provide a conservative
/// (i.e., not underestimated) token count.
///
/// For a more accurate count, a proper tokenizer for the target LLM (e.g., `tiktoken`)
/// should be used.
pub fn estimate(text: &str) -> TokenCount {
    if text.is_empty() {
        return TokenCount(0);
    }

    // Estimate based on characters. The factor of 4 is a common rule of thumb.
    let char_based_estimate = (text.len() as f64 / 4.0).ceil() as usize;

    // Estimate based on words. The 4/3 factor is another common rule of thumb (100 tokens ~ 75 words).
    let word_count = text.split_whitespace().count();
    let word_based_estimate = (word_count as f64 * 4.0 / 3.0).ceil() as usize;

    // Use the maximum of the two heuristics for a conservative estimate.
    TokenCount(char_based_estimate.max(word_based_estimate))
}
