use std::collections::HashMap;

pub(crate) fn categorise_lines(data: &str) -> HashMap<&str, Vec<&str>> {
    let mut current_section = None;
    let mut sections: HashMap<&str, Vec<&str>> = HashMap::new();
    for line in data.lines() {
        if line.starts_with('#') {
            continue;
        }

        if let Some(section) = current_section {
            if line == "end" {
                current_section = None;
            } else {
                sections.get_mut(section).unwrap().push(line);
            }
        } else {
            current_section = Some(line);
            sections.insert(line, vec![]);
        }
    }
    sections
}
