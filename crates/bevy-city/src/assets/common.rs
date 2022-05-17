use std::collections::HashMap;

pub fn categorise_lines(data: &str) -> HashMap<&str, Vec<&str>> {
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

pub fn split_line(line: &str) -> Vec<&str> {
    line.split_ascii_whitespace()
        .map(|s| s.trim().trim_matches(','))
        .collect()
}

mod tests {
    pub use super::*;

    #[test]
    fn can_split_line_with_mix_of_commas_and_spaces() {
        let split = split_line("Id, ModelName, TxdName, MeshCount    DrawDistance, Flags");
        assert_eq!(
            split,
            vec![
                "Id",
                "ModelName",
                "TxdName",
                "MeshCount",
                "DrawDistance",
                "Flags",
            ],
        );
    }
}
