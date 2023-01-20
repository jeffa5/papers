use std::{
    fmt::Debug,
    io::{self, stdout, BufRead, Write},
    str::FromStr,
};

/// Get a line of input as provided.
pub fn input_string(prompt: &str) -> String {
    let mut stdin = io::stdin().lock();

    print!("{}: ", prompt);
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    input.trim().to_owned()
}

/// Get a line of input converted to a FromStr type.
pub fn input<T: FromStr + Debug>(prompt: &str) -> T
where
    <T as FromStr>::Err: Debug,
{
    let input = input_string(prompt);
    T::from_str(&input).unwrap()
}

/// Get a line of input converted to a FromStr type, or the default value.
pub fn input_default<T: FromStr + Debug + Default + PartialEq>(prompt: &str, default: &str) -> T
where
    <T as FromStr>::Err: Debug,
{
    let input = input_string(&format!("{} [{}]", prompt, default));
    match T::from_str(&input) {
        Ok(res) => {
            if res != T::default() {
                res
            } else {
                T::from_str(default).unwrap()
            }
        }
        Err(_) => T::from_str(default).unwrap(),
    }
}

/// Get a line of input converted to a FromStr type if there was any.
pub fn input_opt<T: FromStr + Debug>(prompt: &str) -> Option<T>
where
    <T as FromStr>::Err: Debug,
{
    let input = input_string(&format!("{} (optional)", prompt));
    if input.is_empty() {
        None
    } else {
        Some(T::from_str(&input).unwrap())
    }
}

/// Get a line of input converted to a FromStr type if there was any.
pub fn input_vec_default<T: FromStr + Debug>(prompt: &str, sep: &str, default: &str) -> Vec<T>
where
    <T as FromStr>::Err: Debug,
{
    let input = input_vec(&format!("{} [{}]", prompt, default), sep);
    if input.is_empty() {
        default
            .split(sep)
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(T::from_str(s).unwrap())
                }
            })
            .collect()
    } else {
        input
    }
}

/// Get a line of input converted to a FromStr type if there was any.
pub fn input_vec<T: FromStr + Debug>(prompt: &str, sep: &str) -> Vec<T>
where
    <T as FromStr>::Err: Debug,
{
    let input = input_string(&format!("{} (separated by '{}')", prompt, sep));
    input
        .split(sep)
        .filter_map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(T::from_str(s).unwrap())
            }
        })
        .collect()
}

/// Get a line of input converted to a FromStr type if there was any.
pub fn input_bool(prompt: &str, default: bool) -> bool {
    let input = input_string(&format!(
        "{} [{}/{}]",
        prompt,
        if default { "Y" } else { "y" },
        if default { "n" } else { "N" }
    ));
    match input.to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    }
}
