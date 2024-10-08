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

impl FitHeader {
    pub fn new(
        header_size: u8,
        protocol_version: u8,
        profile_version_lsb: u8,
        profile_version_msb: u8,
        data_size_lsb: u8,
        data_size: u16,
        data_size_msb: u8,
        data_type: u32,
        crc_lsb: u8,
        crc_msb: u8,
    ) -> Result<FitHeader, io::Error> {
        if data_type != 0x5449462e {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid data_type, is this a .fit file?",
            ));
        }

        Ok(FitHeader {
            header_size,
            protocol_version,
            profile_version_lsb,
            profile_version_msb,
            data_size_lsb,
            data_size,
            data_size_msb,
            data_type,
            crc_lsb,
            crc_msb,
        })
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<FitHeader, io::Error> {
        Ok(FitHeader::new(
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u16::<LittleEndian>()?,
            reader.read_u8()?,
            reader.read_u32::<LittleEndian>()?,
            reader.read_u8()?,
            reader.read_u8()?,
        )?)
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

#[derive(Debug)]
pub struct NormalHeader {
    raw: u8,
    local_message_type: u8,
    reserved: u8,
    message_type_spec: u8,
    // 1 -> DefinitionMessage 0 -> DataMessage
    message_type: u8,
    header_type: u8,
}

#[derive(Debug)]
pub struct CompressedTimestampHeader {
    raw: u8,
    time_offset: u8,
    local_message_type: u8,
    header_type: u8,
}

#[derive(Debug)]
pub enum RecordHeader {
    NormalHeader(NormalHeader),
    CompressedTimestampHeader(CompressedTimestampHeader),
}

impl RecordHeader {
    pub fn new(raw: u8) -> Result<RecordHeader, &'static str> {
        let header_type = (raw >> 7) & 0b0000_0001;

        match header_type {
            0 => Ok(RecordHeader::NormalHeader(NormalHeader {
                raw: raw,
                local_message_type: raw & 0b0000_1111,
                reserved: (raw >> 4) & 0b0000_0001,
                message_type_spec: (raw >> 5) & 0b0000_0001,
                message_type: (raw >> 6) & 0b0000_0001,
                header_type: header_type,
            })),
            1 => Ok(RecordHeader::CompressedTimestampHeader(
                CompressedTimestampHeader {
                    raw: raw,
                    time_offset: raw & 0b0001_1111,
                    local_message_type: (raw >> 5) & 0b0000_0011,
                    header_type: header_type,
                },
            )),
            _ => Err("Invalid header type"),
        }
    }
}

#[derive(Debug)]
pub struct DefinitionMessage {
    raw_record_header: u8,
    record_header: RecordHeader,
    reserved: Option<u8>,
    architecture: Option<u8>,
    global_message_number: Option<u16>,
    num_fields: Option<u8>,
    timestamp: Option<u32>,
    //Should probably be parsed as Vec<[u8;3]> as each component contains different info
    field_definitions: Option<Vec<[u8; 3]>>,
    //If the RecordHeader is of NormalHeader Type and has the developer bit(message_type_spec) set
    //the DefinitionMessage also contains developer field definitions, the number of which need to
    //be parsed as the first byte after the field_definitions. Normally we expect num_dev_fields to
    //be set to zero
    num_dev_fields: u8,
    //Should probably be parsed as Vec<[u8;3]> as each component contains different info
    dev_field_definitions: Option<Vec<[u8; 3]>>,
}

#[derive(Debug)]
pub struct DataMessage {
    raw_record_header: u8,
    record_header: RecordHeader,
    data: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum FitRecord {
    DefinitionMessage(DefinitionMessage),
    DataMessage(DataMessage),
}

impl FitRecord {
    pub fn new(raw: &u8) -> Result<FitRecord, &'static str> {
        let header_type = (raw >> 7) & 0b0000_0001;
        let message_type = (raw >> 6) & 0b0000_0001;
        let message_type_spec = (raw >> 5) & 0b0000_0001;

        match header_type {
            0 => {
                match message_type {
                    1 => {
                        Ok(FitRecord::DefinitionMessage(DefinitionMessage {
                            raw_record_header: *raw,
                            record_header: RecordHeader::new(*raw)
                                .expect("Could not create RecordHeader"),
                            reserved: None,
                            architecture: None,
                            global_message_number: None,
                            num_fields: None,
                            timestamp: None,
                            field_definitions: None,
                            //Sets to zero if no dev flag, otherwise one
                            num_dev_fields: message_type_spec,
                            dev_field_definitions: None,
                        }))
                    }
                    0 => Ok(FitRecord::DataMessage(DataMessage {
                        raw_record_header: *raw,
                        record_header: RecordHeader::new(*raw)
                            .expect("Could not create RecordHeader"),
                        data: None,
                    })),
                    _ => Err("Parsing of message type failed"),
                }
            }
            1 => Ok(FitRecord::DataMessage(DataMessage {
                raw_record_header: *raw,
                record_header: RecordHeader::new(*raw).expect("Could not create RecordHeader"),
                data: None,
            })),
            _ => Err("Parsing of header type failed"),
        }
    }
}

#[derive(Debug)]
pub struct FitFile {
    pub header: FitHeader,
    pub data: Vec<FitRecord>,
}
impl FitFile {
    pub fn new(file_path: &str) -> io::Result<FitFile> {
        let mut file = File::open(file_path)?;
        let header = FitHeader::from_reader(&mut file)?;
        println!("{:?}", header);
        file.seek(SeekFrom::Start(14))?;
        let mut buffer = vec![0u8; 1];
        file.read_exact(&mut buffer)?;

        let mut data: Vec<FitRecord> = Vec::new();
        if let Ok(fr) = FitRecord::new(&buffer[0]) {
            println!("{:?}", fr);
            data.push(fr);
        } else {
            println!("Failed to craete FitRecord")
        }

        Ok(FitFile { header, data })
    }
}
