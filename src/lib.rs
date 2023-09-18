use std::collections::HashMap;
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::str;
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Entry {
    flags: u32,
    pub filename: String,
    pub uncompressed_size: u32,
    pub compressed_size: u32,
    pub offset: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
}

pub struct Archive<R: ?Sized + Read + Seek> {
    pub reader: Box<R>,
    pub entries: HashMap<String, Entry>,
}

impl<R: ?Sized + Read + Seek> Archive<R> {
    pub fn from_reader(mut reader: Box<R>) -> io::Result<Self> {
        let entry_count = reader.read_u32::<LittleEndian>()?;
        let mut entries = HashMap::new();

        for _ in 0..entry_count {
            let flags = reader.read_u32::<LittleEndian>()?;

            let mut filename_buf: [u8; 128] = [0; 128];
            reader.read_exact(&mut filename_buf).expect("Failed to read entry bytes");

            let filename_endpos = filename_buf.iter().position(|&c| c == 0).unwrap();
            let (filename_bytes, _) = filename_buf.split_at(filename_endpos);
            let filename = str::from_utf8(&filename_bytes).unwrap();

            let uncompressed_size = reader.read_u32::<LittleEndian>()?;
            let compressed_size = reader.read_u32::<LittleEndian>()?;
            let offset = reader.read_u32::<LittleEndian>()?;
            let unk2 = reader.read_u32::<LittleEndian>()?;
            let unk3 = reader.read_u32::<LittleEndian>()?;
            let unk4 = reader.read_u32::<LittleEndian>()?;

            entries.insert(String::from(filename), Entry {
                flags,
                filename: String::from(filename),
                uncompressed_size,
                compressed_size,
                offset,
                unk2,
                unk3,
                unk4,
            });
        }

        Ok(Archive {
            reader,
            entries
        })
    }

    pub fn extract(&mut self, filename: &str) -> io::Result<Vec<u8>> {
        let entry = self.entries.get(filename)
            .expect("File not found");
        let ret_size = usize::try_from(entry.uncompressed_size)
            .expect("Uncompressed size overflow");
        let mut ret = vec![0u8; ret_size];

        self.reader.seek(SeekFrom::Start(entry.offset.into()))
            .expect("Failed to seek");
        if entry.uncompressed_size != entry.compressed_size {
            let mut decoder = ZlibDecoder::new(&mut self.reader);
            decoder.read_exact(&mut ret)
                .expect("Failed to decompress data");
        } else {
            self.reader.read_exact(&mut ret)
                .expect("Failed to read data");
        }
        Ok(ret)
    }
}