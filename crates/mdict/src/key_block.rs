use byteorder::{LittleEndian, WriteBytesExt};
use flate2::read::ZlibDecoder;
use nom::{
    combinator::{cond, map, map_res},
    error::ParseError,
    multi::{count, length_count},
    number::streaming::{be_u16, be_u64, be_u8, le_u16, le_u32, le_u8},
    sequence::tuple,
    AsBytes, IResult, InputIter, InputLength, Parser, Slice,
};
use ripemd128::{Digest, Ripemd128};
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    ops::RangeFrom,
};

use crate::{
    common::{cond_if, mdict_number, mdict_string},
    content_block,
    dict_meta::DictMeta,
    NomResult, Result,
};

#[derive(Debug)]
struct KeyBlockHeader {
    n_blocks: u64,
    n_entries: u64,
    nb_decompressed: Option<u64>,
    nb_block_info: u64,
    nb_blocks: u64,
    checksum: Option<u32>,
}

fn key_block_header<I, E>(meta: &DictMeta) -> impl FnMut(I) -> IResult<I, KeyBlockHeader, E>
where
    I: Clone + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    E: ParseError<I>,
{
    map(
        tuple((
            mdict_number(meta),
            mdict_number(meta),
            cond(meta.is_ver2(), be_u64),
            mdict_number(meta),
            mdict_number(meta),
            cond(meta.is_ver2(), le_u32),
        )),
        |(n_blocks, n_entries, nb_decompressed, nb_block_info, nb_blocks, checksum)| {
            KeyBlockHeader {
                n_entries,
                n_blocks,
                nb_decompressed,
                nb_block_info,
                nb_blocks,
                checksum,
            }
        },
    )
}

#[derive(Debug)]
struct KeyBlockInfo {
    n_entries: u64,
    head: String,
    tail: String,
    nb_compressed: u64,
    nb_decompressed: u64,
}

fn info_unzip(in_: Vec<u8>, checksum: u32) -> Result<Vec<u8>> {
    let key: Vec<u8>;
    {
        let mut vec = Vec::with_capacity(8);
        vec.write_u32::<LittleEndian>(checksum)?;
        vec.write_u32::<LittleEndian>(0x3695)?;

        let mut hasher = Ripemd128::new();
        hasher.input(vec);
        key = hasher.result().to_vec();
    }

    let mut prev = 0x36;
    let in_ = in_
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let mut t = (*b >> 4 | *b << 4) & 0xff;
            t = t ^ prev ^ (i & 0xff) as u8 ^ key[i % key.len()];

            prev = *b;
            t
        })
        .collect::<Vec<u8>>();

    let mut output = Vec::new();

    {
        let mut decoder = ZlibDecoder::new(Cursor::new(in_));
        decoder.read_to_end(&mut output)?;
    }

    Ok(output)
}

fn info_key<I, E>(meta: &DictMeta) -> impl FnMut(I) -> IResult<I, String, E>
where
    I: Clone + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    E: ParseError<I>,
{
    let is_ver2 = meta.is_ver2();
    let is_utf8 = meta.encoding == "UTF-8";

    fn key_bytes<I, O, E, F>(is_ver2: bool, f: F) -> impl Parser<I, Vec<O>, E>
    where
        I: Clone + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
        F: Parser<I, O, E>,
        E: ParseError<I>,
    {
        map(
            length_count(
                map(
                    cond_if(is_ver2, be_u16, map(be_u8, |v| v as u16)),
                    move |v| {
                        if is_ver2 {
                            v + 1
                        } else {
                            v
                        }
                    },
                ),
                f,
            ),
            move |mut v| {
                if is_ver2 {
                    v.truncate(v.len() - 1);
                }
                v
            },
        )
    }

    cond_if(
        is_utf8,
        map(key_bytes(is_ver2, le_u8), |v| {
            String::from_utf8(v).unwrap_or_default()
        }),
        map(key_bytes(is_ver2, le_u16), |v| {
            String::from_utf16(&v).unwrap_or_default()
        }),
    )
}

fn info_normal<I, E>(
    meta: &DictMeta,
    header: &KeyBlockHeader,
) -> impl FnMut(I) -> NomResult<I, Vec<KeyBlockInfo>, E>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    E: ParseError<I>,
{
    count(
        map(
            tuple((
                mdict_number(meta),
                info_key(meta),
                info_key(meta),
                mdict_number(meta),
                mdict_number(meta),
            )),
            |(n_entries, head, tail, nb_compressed, nb_decompressed): (
                u64,
                String,
                String,
                u64,
                u64,
            )| KeyBlockInfo {
                n_entries,
                head,
                tail,
                // 不包含 type 和 checksum
                nb_compressed: nb_compressed - 8,
                nb_decompressed,
            },
        ),
        header.n_blocks as usize,
    )
}

fn key_block_info<I>(
    in_: I,
    meta: &DictMeta,
    header: &KeyBlockHeader,
) -> NomResult<I, Vec<KeyBlockInfo>>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    if meta.is_ver2() {
        let (in_, (_, checksum)) = tuple((le_u32, le_u32))(in_)?;
        let (in_, data) = map_res(count(le_u8, header.nb_block_info as usize - 8), |data| {
            info_unzip(data, checksum)
        })(in_)?;

        let (_, infos) = info_normal(meta, header)(data.as_slice())?;
        Ok((in_, infos))
    } else {
        info_normal(meta, header)(in_)
    }
}

type KeyMap = HashMap<String, u64>;

fn key_map<I>(
    mut in_: I,
    meta: &DictMeta,
    header: &KeyBlockHeader,
    infos: &Vec<KeyBlockInfo>,
) -> NomResult<I, KeyMap>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let mut keymap = KeyMap::with_capacity(header.n_entries as usize);

    for item in infos {
        let (i_, data) = content_block::parse(in_, item.nb_compressed, item.nb_decompressed)?;
        in_ = i_;

        let (_, entries) = count(
            tuple((mdict_number(meta), mdict_string(meta))),
            item.n_entries as usize,
        )(data.as_bytes())?;

        entries.iter().for_each(|entry| {
            keymap.insert(entry.1.clone(), entry.0);
        })
    }

    println!("{:?}", keymap.iter().take(4).collect::<Vec<_>>());
    Ok((in_, keymap))
}

#[derive(Debug)]
pub struct KeyBlock {
    header: KeyBlockHeader,
    infos: Vec<KeyBlockInfo>,
    pub keymap: KeyMap,
}

pub fn parse<I>(in_: I, meta: &DictMeta) -> NomResult<I, KeyBlock>
where
    I: Clone + PartialEq + Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    let (in_, header) = key_block_header(meta)(in_)?;
    println!("{:?}", header);

    let (in_, infos) = key_block_info(in_, meta, &header)?;

    let (in_, keymap) = key_map(in_, meta, &header, &infos)?;

    Ok((
        in_,
        KeyBlock {
            header,
            infos,
            keymap,
        },
    ))
}
