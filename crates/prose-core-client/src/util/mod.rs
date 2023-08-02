use std::ops::Deref;

pub fn concatenate_names(
    first_name: &Option<String>,
    last_name: &Option<String>,
) -> Option<String> {
    let parts = first_name
        .iter()
        .chain(last_name.iter())
        .map(|s| s.deref())
        .collect::<Vec<_>>();

    (!parts.is_empty())
        .then_some(parts)
        .map(|parts| parts.join(" "))
}
