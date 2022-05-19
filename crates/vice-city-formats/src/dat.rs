#[derive(PartialEq, Eq, Debug)]
pub struct GtaVcDat {
    pub ides: Vec<String>,
    pub ipls: Vec<String>,
}

pub fn parse_gta_vc_dat(dat: &str) -> GtaVcDat {
    let mut ides = vec!["data/default.ide".to_string()];
    let mut ipls = vec![];

    for (filetype, path) in dat
        .lines()
        .filter(|l| !(l.trim().is_empty() || l.starts_with('#')))
        .filter_map(|l| l.split_once(' '))
    {
        if !path.starts_with("DATA\\MAPS") {
            continue;
        }

        let path = path
            .replace('\\', "/")
            .replace("DATA/MAPS", "data/maps")
            .replace(".IDE", ".ide")
            .replace(".IPL", ".ipl")
            // hack: fix the case on some map IDEs...
            .replace("haitin/haitin.ide", "haitiN/haitiN.ide")
            .replace("oceandn/oceandn", "oceandn/oceandN")
            // hack: fix the case on some map IDLs...
            .replace("club.ipl", "CLUB.ipl")
            .replace("haitin/haitin.ipl", "haitiN/haitin.ipl");

        // hack: remove some ipls we don't care for
        if path.ends_with("islandsf.ipl") {
            continue;
        }

        match filetype {
            "IDE" => ides.push(path),
            "IPL" => ipls.push(path),
            _ => {}
        }
    }

    GtaVcDat { ides, ipls }
}

pub mod tests {
    #[test]
    fn can_parse_subset_of_gta_vc_dat() {
        const TEST_DATA: &str = r#"
# Load IDEs first, then the models and after that the IPLs

IDE DATA\MAPS\stadint\stadint.IDE
COLFILE 0 MODELS\COLL\GENERIC.COL
SPLASH loadsc3
IPL DATA\MAPS\downtown\downtown.IPL
IPL DATA\MAPS\haitin\haitin.IPL
        "#;

        let test_data = TEST_DATA.trim();
        assert_eq!(
            super::parse_gta_vc_dat(test_data),
            super::GtaVcDat {
                ides: vec![
                    "data/default.ide".to_string(),
                    "data/maps/stadint/stadint.ide".to_string()
                ],
                ipls: vec![
                    "data/maps/downtown/downtown.ipl".to_string(),
                    "data/maps/haitiN/haitin.ipl".to_string()
                ]
            }
        );
    }
}
