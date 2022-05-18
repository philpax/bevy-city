use std::collections::HashMap;

pub fn categorise_lines(data: &str) -> HashMap<&str, Vec<&str>> {
    let mut current_section = None;
    let mut sections: HashMap<&str, Vec<&str>> = HashMap::new();
    for line in data.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
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
    fn can_categorise_some_lines() {
        const TEST_DATA: &str = r#"
peds
9, HFYST, HFYST, CIVFEMALE, STAT_STREET_GIRL, sexywoman, 013, null, 6,1		
83, CBa, CBa, GANG1, STAT_GANG1, gang1, 0, null, 6,6
end

cars
130,	landstal, 	landstal, 	car, 	LANDSTAL, 	LANDSTK, 		null,	normal, 	10,	7,	0,		254, 0.8
end

"#;

        let test_data = TEST_DATA.trim();
        assert_eq!(categorise_lines(test_data), HashMap::from([
            ("peds", vec![
                "9, HFYST, HFYST, CIVFEMALE, STAT_STREET_GIRL, sexywoman, 013, null, 6,1\t\t",
                "83, CBa, CBa, GANG1, STAT_GANG1, gang1, 0, null, 6,6",
            ]),
            ("cars", vec![
                "130,\tlandstal, \tlandstal, \tcar, \tLANDSTAL, \tLANDSTK, \t\tnull,\tnormal, \t10,\t7,\t0,\t\t254, 0.8",
            ])
        ]));
    }

    #[test]
    fn can_split_line_with_mix_of_commas_and_spaces() {
        let split = split_line("Id, ModelName, TxdName,    MeshCount\t\tDrawDistance, Flags");
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
