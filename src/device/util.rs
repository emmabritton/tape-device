pub fn convert_and_fit(parts: Vec<&str>, max_width: usize, padding: &str) -> Vec<String> {
    fit_in_lines(
        parts
            .iter()
            .map(|txt| format!("{}{}", txt, padding))
            .collect(),
        max_width,
    )
}

pub fn fit_in_lines(parts: Vec<String>, max_width: usize) -> Vec<String> {
    let mut output = vec![];

    for part in parts {
        if output
            .last()
            .map(|str: &String| str.chars().count())
            .unwrap_or(0)
            + part.chars().count()
            < max_width
        {
            if output.is_empty() {
                output.push(part);
            } else {
                output.last_mut().unwrap().push_str(&part)
            }
        } else {
            let lines = breaklines(part.clone(), max_width);
            output.extend_from_slice(&lines);
        }
    }

    output
}

pub fn breaklines(mut line: String, max_width: usize) -> Vec<String> {
    return if line.chars().count() < max_width {
        vec![line]
    } else {
        let mut output = vec![];
        while !line.is_empty() {
            let temp = line.trim_start();
            let mut idx =
                index_of_first_non_alphanumeric_before(temp, max_width - 2).unwrap_or(max_width);
            if idx == 0 {
                idx = max_width.min(temp.chars().count());
            }
            let first: String = temp.chars().take(idx).collect();
            let temp: String = temp.chars().skip(idx).collect();
            output.push(first.trim().to_string());
            if temp.chars().count() >= max_width {
                line = temp.to_string();
            } else {
                let new_line = temp.trim();
                if !new_line.is_empty() {
                    output.push(new_line.to_string());
                }
                line.clear();
            }
        }
        output
    };
}

fn index_of_first_non_alphanumeric_before(str: &str, start: usize) -> Option<usize> {
    let chars: Vec<char> = str.chars().collect();
    index_of_first_before(&mut chars.iter(), |chr| !chr.is_ascii_alphanumeric(), start)
}

fn index_of_first_before<I, T, P>(iter: &mut I, pred: P, start: usize) -> Option<usize>
where
    I: ExactSizeIterator<Item = T> + std::iter::DoubleEndedIterator,
    P: FnMut(T) -> bool,
    T: PartialEq,
{
    iter.take(start).rposition(pred)
}

#[cfg(test)]
mod test {
    use super::*;

    mod public {
        use super::*;

        #[test]
        fn test_fit_in_lines() {
            let list = vec![
                String::from("Some words"),
                String::from("A_long_word"),
                String::from("REG: x45"),
                String::from("Short"),
                String::from("Test breaklines is executed correctly"),
            ];
            let result1 = fit_in_lines(list.clone(), 4);
            let result2 = fit_in_lines(list.clone(), 8);
            let result3 = fit_in_lines(list, 15);

            assert_eq!(
                result1,
                vec![
                    String::from("Some"),
                    String::from("word"),
                    String::from("s"),
                    String::from("A"),
                    String::from("_lon"),
                    String::from("g"),
                    String::from("_wor"),
                    String::from("d"),
                    String::from("REG:"),
                    String::from("x45"),
                    String::from("Shor"),
                    String::from("t"),
                    String::from("Test"),
                    String::from("brea"),
                    String::from("klin"),
                    String::from("es i"),
                    String::from("s"),
                    String::from("exec"),
                    String::from("uted"),
                    String::from("corr"),
                    String::from("ectl"),
                    String::from("y")
                ]
            );
            assert_eq!(
                result2,
                vec![
                    String::from("Some"),
                    String::from("words"),
                    String::from("A"),
                    String::from("_long"),
                    String::from("_word"),
                    String::from("REG:"),
                    String::from("x45"),
                    String::from("Short"),
                    String::from("Test"),
                    String::from("breaklin"),
                    String::from("es is"),
                    String::from("executed"),
                    String::from("correctl"),
                    String::from("y")
                ]
            );
            assert_eq!(
                result3,
                vec![
                    String::from("Some words"),
                    String::from("A_long_word"),
                    String::from("REG: x45Short"),
                    String::from("Test"),
                    String::from("breaklines"),
                    String::from("is executed"),
                    String::from("correctly")
                ]
            );
        }

        #[test]
        fn test_breaklines() {
            let str = String::from(
                "This is a long test string, used for testing how lines are broken by breaklines",
            );
            let result1 = breaklines(str.clone(), 4);
            let result2 = breaklines(str.clone(), 10);
            let result3 = breaklines(str, 20);

            assert_eq!(
                result1,
                vec![
                    String::from("This"),
                    String::from("is a"),
                    String::from("long"),
                    String::from("test"),
                    String::from("stri"),
                    String::from("ng,"),
                    String::from("used"),
                    String::from("for"),
                    String::from("test"),
                    String::from("ing"),
                    String::from("how"),
                    String::from("line"),
                    String::from("s"),
                    String::from("are"),
                    String::from("brok"),
                    String::from("en b"),
                    String::from("y"),
                    String::from("brea"),
                    String::from("klin"),
                    String::from("es")
                ]
            );
            assert_eq!(
                result2,
                vec![
                    String::from("This is"),
                    String::from("a long"),
                    String::from("test"),
                    String::from("string,"),
                    String::from("used"),
                    String::from("for"),
                    String::from("testing"),
                    String::from("how"),
                    String::from("lines"),
                    String::from("are"),
                    String::from("broken"),
                    String::from("by"),
                    String::from("breaklines")
                ]
            );
            assert_eq!(
                result3,
                vec![
                    String::from("This is a long"),
                    String::from("test string, used"),
                    String::from("for testing how"),
                    String::from("lines are broken"),
                    String::from("by breaklines")
                ]
            );
        }

        #[test]
        fn test_breaklines_no_whitespace() {
            let str = String::from("Stardenburdenhardenbart");
            let result = breaklines(str, 8);
            assert_eq!(
                result,
                vec![
                    String::from("Stardenb"),
                    String::from("urdenhar"),
                    String::from("denbart")
                ]
            );
        }

        #[test]
        fn test_breaklines_symbols() {
            let str = String::from("A_long_test_word");
            let result = breaklines(str, 5);
            assert_eq!(
                result,
                vec![
                    String::from("A"),
                    String::from("_long"),
                    String::from("_test"),
                    String::from("_word")
                ]
            );
        }
    }

    mod internal {
        use super::*;

        #[test]
        fn test_index_of_first_non_alphanumeric_before() {
            let str = "this is a test string";
            let result1 = index_of_first_non_alphanumeric_before(str, str.chars().count());
            let result2 = index_of_first_non_alphanumeric_before(str, 10);
            let result3 = index_of_first_non_alphanumeric_before(str, 9);
            let result4 = index_of_first_non_alphanumeric_before(str, 6);
            let result5 = index_of_first_non_alphanumeric_before(str, 2);

            assert_eq!(result1, Some(14));
            assert_eq!(result2, Some(9));
            assert_eq!(result3, Some(7));
            assert_eq!(result4, Some(4));
            assert_eq!(result5, None);
        }

        #[test]
        fn test_index_of_first_before() {
            let list = vec![1, 2, 3];
            let result1 = index_of_first_before(&mut list.iter(), |num| num == &2, 0);
            let result2 = index_of_first_before(&mut list.iter(), |num| num == &2, 2);

            assert_eq!(result1, None);
            assert_eq!(result2, Some(1));

            let list = vec!['a', 'k', 'p', 'n'];
            let result1 = index_of_first_before(&mut list.iter(), |chr| chr == &'n', 0);
            let result2 = index_of_first_before(&mut list.iter(), |chr| chr == &'p', 2);
            let result3 = index_of_first_before(&mut list.iter(), |chr| chr == &'p', 3);

            assert_eq!(result1, None);
            assert_eq!(result2, None);
            assert_eq!(result3, Some(2));
        }
    }
}
