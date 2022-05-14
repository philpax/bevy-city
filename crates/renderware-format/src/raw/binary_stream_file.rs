use std::{fs::File, io, path::Path};

use super::{Error, Section, UnparsedData};

use nom::Finish;

#[derive(Debug, PartialEq)]
pub struct BinaryStreamFile {
    pub sections: Vec<Section>,
}

impl BinaryStreamFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_reader(io::BufReader::new(File::open(path)?))
    }

    pub fn from_reader<R: io::Read + io::Seek + Send + 'static>(
        mut reader: R,
    ) -> Result<Self, Error> {
        let mut data = vec![];
        reader.read_to_end(&mut data)?;
        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
        // We only support one section for now
        let (_, section) = Section::parse(data, None)
            .map_err(|err| err.map_input(|i| UnparsedData(i.to_owned())))
            .finish()?;

        Ok(BinaryStreamFile {
            sections: vec![section],
        })
    }
}
