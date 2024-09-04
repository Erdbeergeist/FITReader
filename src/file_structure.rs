use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct FitHeader {
    pub header_size: u8,
    pub protocol_version: u8,
    pub profile_version_lsb: u8,
    pub profile_version_msb: u8,
    pub data_size_lsb: u8,
    pub data_size: u16,
    pub data_size_msb: u8,
    pub data_type: u32,
    pub crc_lsb: u8,
    pub crc_msb: u8,
}

#[derive(Debug)]
pub struct FitFile {
    pub header: FitHeader,
    pub data: Vec<u8>,
}

impl FitHeader {
    pub fn from_reader<R: Read>(reader: &mut R) -> io::Result<FitHeader> {
        Ok(FitHeader {
            header_size: reader.read_u8()?,
            protocol_version: reader.read_u8()?,
            profile_version_lsb: reader.read_u8()?,
            profile_version_msb: reader.read_u8()?,
            data_size_lsb: reader.read_u8()?,
            data_size: reader.read_u16::<LittleEndian>()?,
            data_size_msb: reader.read_u8()?,
            data_type: reader.read_u32::<LittleEndian>()?,
            crc_lsb: reader.read_u8()?,
            crc_msb: reader.read_u8()?,
        })
    }

    pub fn pretty_print(&self) {
        let bytes = self.data_type.to_le_bytes();
        let data_type_chars: String = bytes
            .iter()
            .map(|&b| std::char::from_u32(b as u32).unwrap_or('?'))
            .collect();

        println!("Data Type: {}", data_type_chars);
    }
}

impl FitFile {
    pub fn from_file(file_path: &str) -> io::Result<FitFile> {
        let mut file = File::open(file_path)?;
        let mut header_bytes = [0, 14];

        file.read_exact(&mut header_bytes)?;
        let mut reader = &header_bytes[..];
        let header = FitHeader::from_reader(&mut reader)?;

        let mut data = vec![0, 1];

        Ok(FitFile { header, data })
    }
}
