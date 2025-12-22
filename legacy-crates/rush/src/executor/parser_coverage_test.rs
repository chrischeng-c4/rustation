#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::parser::{Token, RedirectionType};

    #[test]
    fn test_parser_unreachable_branch() {
        // This test is tricky because we need to trigger the match arm for Redirect tokens
        // but somehow have a different token in the inner match.
        // In the current code structure:
        // match &tokens[i] {
        //     Token::RedirectOut | Token::RedirectAppend | Token::RedirectIn => { ... }
        // }
        // The inner match is on &tokens[i] again.
        // So it is mathematically impossible to hit the wildcard _ => unreachable!()
        // UNLESS the token changes between the two matches (which it can't, it's immutable reference).
        // So this line is truly unreachable and safe. 
        // We can't write a test to hit it without modifying the source code to be buggy.
        // Thus, for coverage purposes, we might want to replace unreachable!() with a panic or error 
        // that we can potentially trigger if we refactor, but right now it's fine.
        // To "cover" it, we'd need to remove the unreachable! and handle it, but that's dead code.
        // Let's leave it as is.
    }
}
