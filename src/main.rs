use std::{
    fs::File,
    io::{self, BufReader, Cursor, Read},
};

use inflate::InflateStream;

struct ParsedData {
    header: String,
    ver: i32,
    width: i32,
    height: i32,
    tags: i32,
    tag_strings: Vec<(String, String)>,
    blocks: Vec<String>,
    total_blocks: i32,
    block_data: Vec<(i32, i32, i32, i32)>,
}

fn read_u8(cursor: &mut Cursor<&Vec<u8>>) -> io::Result<u8> {
    let mut buf = [0; 1];
    cursor.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16(cursor: &mut Cursor<&Vec<u8>>) -> io::Result<u16> {
    let mut buf = [0; 2];
    cursor.read_exact(&mut buf)?;
    Ok(((buf[0] as u16) << 8) | buf[1] as u16)
}

fn read_string(cursor: &mut Cursor<&Vec<u8>>) -> io::Result<String> {
    let mut string_size_buffer_tag = [0; 2];
    cursor.read_exact(&mut string_size_buffer_tag)?;
    let string_size = ((string_size_buffer_tag[0] as u32) << 8) | string_size_buffer_tag[1] as u32;

    let mut string_buf_tag: Vec<u8> = vec![0; string_size as usize];
    cursor.read_exact(&mut string_buf_tag)?;

    String::from_utf8(string_buf_tag)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

fn parse_data(path: &'static str) -> io::Result<ParsedData> {
    let mut data = ParsedData {
        header: String::new(),
        ver: 0,
        width: 0,
        height: 0,
        tags: 0,
        tag_strings: Vec::new(),
        blocks: Vec::new(),
        total_blocks: 0,
        block_data: Vec::new(),
    };

    let input = File::open(path)?;
    let mut reader = BufReader::new(input);

    let mut header = [0; 4];
    reader.read_exact(&mut header)?;
    data.header = String::from_utf8(header.to_vec()).unwrap();

    let mut ver = [0; 3];
    reader.read_exact(&mut ver)?;
    data.ver = 0;

    let mut inflater = InflateStream::new();
    let mut out = Vec::<u8>::new();
    let mut buffer = [0u8; 1024];

    while let Ok(num_bytes_read) = reader.read(&mut buffer) {
        if num_bytes_read == 0 {
            break;
        }

        let res = inflater.update(&buffer[..num_bytes_read]);

        match res {
            Ok((_, result)) => {
                out.extend(result.iter().cloned());
            }
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
            }
        }
    }

    let mut cursor = Cursor::new(&out);

    data.width = read_u16(&mut cursor)? as i32;
    data.height = read_u16(&mut cursor)? as i32;
    data.tags = read_u8(&mut cursor)? as i32;

    for _ in 0..data.tags {
        let label = read_string(&mut cursor)?;
        let content = read_string(&mut cursor)?;
        data.tag_strings.push((label, content));
    }

    let blocks_count = read_u8(&mut cursor)? as i32;

    for _ in 0..blocks_count {
        let name = read_string(&mut cursor)?;
        data.blocks.push(name);
    }

    let total_blocks_count = read_u8(&mut cursor)? as i32;

    for _ in 0..total_blocks_count {
        let name = read_u8(&mut cursor)?;
        let position = read_u16(&mut cursor)?;
        let config = read_u16(&mut cursor)?;
        let rotation = read_u8(&mut cursor)?;
        data.block_data
            .push((name as i32, position as i32, config as i32, rotation as i32));
    }

    Ok(data)
}

fn main() -> io::Result<()> {
    let parsed_data = parse_data("Schematic.msch")?;
    println!("{}", parsed_data.tag_strings[1].1);
    Ok(())
}
