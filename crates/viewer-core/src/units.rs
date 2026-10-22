//! Values and units written readably for people, whoever spelled them:
//! the service (`SI::'kg⋅m⁻³'`), an Engineering Tool (`mm^3`) or the
//! Model (`millimetre ** 3`).

/// A value text with its unit, if it has one in brackets, made readable:
/// `1000000.0 [mm^3]` → `1000000.0 [mm³]`, `default 0.2e6 [mm**3]` →
/// `default 0.2e6 [mm³]`. The rest is kept as written.
pub fn readable_value(text: &str) -> String {
    match (text.find('['), text.rfind(']')) {
        (Some(open), Some(close)) if open < close => format!(
            "{}[{}]{}",
            &text[..open],
            readable_unit(&text[open + 1..close]),
            &text[close + 1..]
        ),
        _ => text.to_owned(),
    }
}

/// A unit as the Model would write it, not as the service spells it:
/// without package prefixes and quotes (`SI::'m³'` → `m³`), with powers as
/// superscripts (`mm**3`, `mm ** 3`, `mm^3` → `mm³`), and a product with
/// negative powers as a quotient (`kg⋅m⁻³` → `kg/m³`).
pub fn readable_unit(unit: &str) -> String {
    let mut plain = String::new();
    let mut chars = unit.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {}
            ':' if chars.peek() == Some(&':') => {
                chars.next();
                // Drops the package name just written.
                while plain.ends_with(|p: char| p.is_alphanumeric() || p == '_') {
                    plain.pop();
                }
            }
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                superscript(&mut chars, &mut plain);
            }
            '^' => superscript(&mut chars, &mut plain),
            c => plain.push(c),
        }
    }
    as_quotient(&plain).unwrap_or(plain)
}

/// Writes the power that follows in `chars` (`-1`, `3`, ` 3`) as
/// superscripts, dropping the spaces around the operator.
fn superscript(chars: &mut std::iter::Peekable<std::str::Chars>, out: &mut String) {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    while out.ends_with(' ') {
        out.pop();
    }
    while chars.peek() == Some(&' ') {
        chars.next();
    }
    if chars.peek() == Some(&'-') {
        chars.next();
        out.push('⁻');
    }
    while let Some(d) = chars.peek().and_then(|c| c.to_digit(10)) {
        chars.next();
        out.push(DIGITS[d as usize]);
    }
}

/// `kg⋅m⁻¹⋅s⁻²` as `kg/(m⋅s²)`; `None` when the unit is no such product,
/// or has no factor above the line (`s⁻¹` stays).
fn as_quotient(unit: &str) -> Option<String> {
    if unit.contains('/') {
        return None;
    }
    let (over, under): (Vec<&str>, Vec<&str>) = unit.split('⋅').partition(|f| !f.contains('⁻'));
    if over.is_empty() || under.is_empty() {
        return None;
    }
    let under: Vec<String> = under
        .iter()
        .map(|f| match f.split_once('⁻') {
            Some((base, "¹")) => base.to_owned(),
            Some((base, power)) => format!("{base}{power}"),
            None => f.to_string(),
        })
        .collect();
    let under = match under.as_slice() {
        [one] => one.clone(),
        many => format!("({})", many.join("⋅")),
    };
    Some(format!("{}/{under}", over.join("⋅")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_have_their_bracketed_unit_made_readable() {
        assert_eq!(readable_value("1000000.0 [mm^3]"), "1000000.0 [mm³]");
        assert_eq!(
            readable_value("900000.0 [millimetre ** 3]"),
            "900000.0 [millimetre³]"
        );
        assert_eq!(
            readable_value("default 0.2e6 [mm**3]"),
            "default 0.2e6 [mm³]"
        );
        assert_eq!(readable_value("7.85"), "7.85");
    }

    #[test]
    fn units_lose_prefixes_and_negative_powers_become_quotients() {
        assert_eq!(readable_unit("SI::'kg⋅m⁻³'"), "kg/m³");
        assert_eq!(readable_unit("kg⋅m^-1⋅s^-2"), "kg/(m⋅s²)");
        assert_eq!(readable_unit("s^-1"), "s⁻¹");
    }
}
