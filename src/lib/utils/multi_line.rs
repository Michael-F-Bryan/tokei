use language::Language;

/// This is used to catch lines like "let x = 5; /* Comment */"
#[inline(never)]
pub fn handle_multi_line(line: &str,
                         language: &Language,
                         stack: &mut Vec<&'static str>,
                         quote: &mut Option<&'static str>) {
    let nested_is_empty = language.nested_comments.is_empty();
    let mut skip = false;

    let window_size = language.multi_line.iter()
        .chain(language.nested_comments.iter())
        .map(|&(first, second)| ::std::cmp::max(first.len(), second.len()))
        .max().unwrap();

    'window: for window in line.as_bytes().windows(window_size) {
        if skip {
            skip = false;
            continue;
        }
        let mut end = false;

        if let &mut Some(quote_str) = quote {
            if window.starts_with(b"\\") {
                continue;
            } else if window.starts_with(quote_str.as_bytes()) {
                end = true;
            }
        }

        if end {
            if let &mut Some(quote_str) = quote {
                *quote = None;

                // Prevent the quote being counted as both a end and start of a quote
                if quote_str.chars().count() == 1 {
                    skip = true;
                }
                continue;
            }
        }

        if quote.is_some() {
            continue;
        }

        let mut pop = false;
        if let Some(last) = stack.last() {
            if window.starts_with(last.as_bytes()) {
                pop = true;
            }
        }

        if pop {
            stack.pop();
            skip = true;
            continue;
        }

        if stack.is_empty() {
            for comment in &language.line_comment {
                if window.starts_with(comment.as_bytes()) {
                    break 'window;
                }
            }

            for &(start, end) in &language.quotes {
                if window.starts_with(start.as_bytes()) {
                    *quote = Some(end);
                    skip = true;
                    continue 'window;
                }
            }
        }

        for &(start, end) in &language.nested_comments {
            if window.starts_with(start.as_bytes()) {
                stack.push(end);
                skip = true;
                continue 'window;
            }
        }

        for &(start, end) in &language.multi_line {
            if window.starts_with(start.as_bytes()) {
                if (language.nested && nested_is_empty) || stack.len() == 0 {
                    stack.push(end);
                }
                skip = true;
                continue 'window;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language::Language;

    #[test]
    fn both_comments_in_line() {
        let mut stack = vec![];
        let mut quote = None;
        let language = Language::new_c();
        handle_multi_line("Hello /* /* */ World", &language, &mut stack, &mut quote);
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn comment_hidden_in_single() {
        let mut stack = vec![];
        let mut quote = None;
        let language = Language::new_c();
        handle_multi_line("Hello World // /*", &language, &mut stack, &mut quote);
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn single_comment_in_multi() {
        let mut stack = vec![];
        let mut quote = None;
        let language = Language::new_c();
        handle_multi_line("Hello /* // */ world", &language, &mut stack, &mut quote);
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn comment_start() {
        let mut stack = vec![];
        let mut quote = None;
        let language = Language::new_c();
        handle_multi_line("/*Hello World", &language, &mut stack, &mut quote);
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn both_comments_in_line_nested() {
        let mut stack = vec![];
        let mut quote = None;
        let language = Language::new_func().nested();
        handle_multi_line("Hello (* (* *) World", &language, &mut stack, &mut quote);
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn comments_of_uneven_length() {
        let mut stack = vec![];
        let mut quote = None;
        let language = Language::new(vec![], vec![("\\<open>", "\\<close>")]).nested();
        handle_multi_line("Hello \\<open> \\<open> \\<close> World",
                          &language,
                          &mut stack,
                          &mut quote);
        assert_eq!(stack.len(), 1);
    }
}
