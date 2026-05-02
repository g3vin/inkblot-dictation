pub fn clean_transcript(raw: &str) -> String {
    let mut text = raw.trim().to_string();

    let replacements = [
        (" new paragraph ", "\n\n"),
        (" new line ", "\n"),
        (" comma", ","),
        (" period", "."),
        (" question mark", "?"),
        (" exclamation point", "!"),
        (" exclamation mark", "!"),
        (" colon", ":"),
        (" semicolon", ";"),
        (" open quote ", " “"),
        (" close quote ", "” "),
        (" quote ", " “"),
        (" end quote ", "” "),
    ];

    text = format!(" {} ", text.to_lowercase());

    for (from, to) in replacements {
        text = text.replace(from, to);
    }

    text = fix_punctuation_spacing(&text);
    text = capitalize_sentences(&text);
    text = capitalize_standalone_i(&text);
    text.trim().to_string()
}

fn fix_punctuation_spacing(input: &str) -> String {
    let mut out = String::new();
    let mut prev_was_space = false;

    for ch in input.chars() {
        if ch == '\n' {
            while out.ends_with(' ') {
                out.pop();
            }

            out.push('\n');
            prev_was_space = false;
        } else if ch.is_whitespace() {
            if !prev_was_space && !out.ends_with('\n') {
                out.push(' ');
            }
            prev_was_space = true;
        } else {
            if matches!(ch, ',' | '.' | '?' | '!' | ':' | ';') {
                while out.ends_with(' ') {
                    out.pop();
                }
            }

            out.push(ch);
            prev_was_space = false;
        }
    }

    out
}

fn capitalize_sentences(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut capitalize_next = true;

    for ch in input.chars() {
        if capitalize_next && ch.is_ascii_alphabetic() {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }

        if matches!(ch, '.' | '?' | '!' | '\n') {
            capitalize_next = true;
        }
    }

    result
}

fn capitalize_standalone_i(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();

    for idx in 0..chars.len() {
        let ch = chars[idx];

        if ch == 'i' {
            let prev_is_boundary = idx == 0 || !chars[idx - 1].is_ascii_alphabetic();
            let next = chars.get(idx + 1).copied();

            let next_is_boundary = match next {
                None => true,
                Some('\'') | Some('’') => true,
                Some(c) => !c.is_ascii_alphabetic(),
            };

            if prev_is_boundary && next_is_boundary {
                result.push('I');
                continue;
            }
        }

        result.push(ch);
    }

    result
}