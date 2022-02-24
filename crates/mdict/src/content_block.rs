use std::{
    io::{Cursor, Read},
    ops::RangeFrom,
};

use flate2::read::ZlibDecoder;
use nom::{
    combinator::map,
    multi::count,
    number::streaming::{le_u32, le_u8},
    sequence::tuple,
    InputIter, InputLength, Slice,
};

use crate::{nom_return, NomResult};

#[derive(Debug)]
enum ContentBlockType {
    UnCompressed = 0,
    LZO = 1,
    Zlib = 2,
}

#[derive(Debug)]
struct ContentBlock {
    block_type: ContentBlockType,
    #[allow(dead_code)]
    checksum: u32,
    data: Vec<u8>,
}

pub fn parse<I>(in_: I, nb_compressed: usize, nb_decompressed: usize) -> NomResult<I, Vec<u8>>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, block) = map(
        tuple((
            map(le_u32, |v| -> ContentBlockType {
                match v {
                    0 => ContentBlockType::UnCompressed,
                    1 => ContentBlockType::LZO,
                    2 => ContentBlockType::Zlib,
                    _ => panic!("{} Unknown ContentBlockType", v),
                }
            }),
            le_u32,
            count(le_u8, nb_compressed - 8),
        )),
        |(block_type, checksum, data)| ContentBlock {
            block_type,
            checksum,
            data,
        },
    )(in_)?;

    nom_return!(in_, Vec<u8>, {
        match block.block_type {
            ContentBlockType::Zlib => {
                let mut output = Vec::with_capacity(nb_decompressed);
                let mut decoder = ZlibDecoder::new(Cursor::new(block.data));
                decoder.read_to_end(&mut output)?;
                output
            }
            ContentBlockType::UnCompressed => block.data,
            ContentBlockType::LZO => {
                let lzo = minilzo_rs::LZO::init()?;

                lzo.decompress(&block.data, nb_decompressed)?
            }
        }
    })
}
