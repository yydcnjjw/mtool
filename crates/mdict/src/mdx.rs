use std::{
    collections::HashMap,
    io::{Cursor, Read},
    ops::RangeFrom,
};

use byteorder::{LittleEndian, WriteBytesExt};

use flate2::read::ZlibDecoder;
use nom::{
    bytes::streaming::tag,
    combinator::{cond, map},
    error::ParseError,
    multi::{count, length_count, many_till},
    number::streaming::{be_u16, be_u32, be_u64, be_u8, le_u16, le_u32, le_u8},
    sequence::tuple,
    AsBytes, Compare, IResult, InputIter, InputLength, InputTake, Parser, Slice,
};
use ripemd128::{Digest, Ripemd128};

use super::{common::cond_if, dict_meta::dict_meta, dict_meta::DictMeta, nom_return, NomResult};

#[derive(Debug)]
pub struct Mdx {
    pub meta: DictMeta,
    pub keymap: KeyMap,
    pub record: Record,
}

impl Mdx {
    pub fn search(&self, text: String) -> Vec<(String, String)> {
        self.keymap
            .iter()
            .filter(|item| item.0.contains(&text))
            .map(|item| (item.0.clone(), self.record.record(&self.meta, *item.1)))
            .collect::<_>()
    }
}

#[derive(Debug)]
pub struct Record {
    pub data: Vec<Vec<u8>>,
}

impl Record {
    fn record(&self, meta: &DictMeta, mut pos: u64) -> String {
        self.data
            .iter()
            .find(|item| {
                let len = item.len() as u64;
                if pos >= len {
                    pos -= len;
                    false
                } else {
                    true
                }
            })
            .map(|item| {
                let r: NomResult<&[u8], String> = mdx_string(meta)(&item[pos as usize..]);
                r.unwrap().1
            })
            .unwrap_or_default()
    }
}

#[derive(Debug)]
struct KeyBlockHeader {
    n_blocks: u64,
    n_entries: u64,
    nb_decompressed: Option<u64>,
    nb_block_info: u64,
    nb_blocks: u64,
    checksum: Option<u32>,
}

fn mdx_number<I, E>(meta: &DictMeta) -> impl FnMut(I) -> IResult<I, u64, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    E: ParseError<I>,
{
    cond_if(meta.is_ver2(), be_u64, map(be_u32, |v| v as u64))
}

const U8NULL: &'static [u8] = &[0u8];
const U16NULL: &'static [u8] = &[0u8, 0u8];

fn mdx_string<I, E>(meta: &DictMeta) -> impl FnMut(I) -> IResult<I, String, E>
where
    I: Clone
        + PartialEq
        + Slice<RangeFrom<usize>>
        + InputIter<Item = u8>
        + InputLength
        + InputTake
        + Compare<&'static [u8]>,
    E: ParseError<I>,
{
    cond_if(
        meta.encoding == "UTF-8",
        map(many_till(le_u8, tag(U8NULL)), |(v, _)| {
            String::from_utf8(v).unwrap_or_default()
        }),
        map(many_till(le_u16, tag(U16NULL)), |(v, _)| {
            String::from_utf16(&v).unwrap_or_default()
        }),
    )
}

fn key_block<'a>(in_: &'a [u8], meta: &DictMeta) -> NomResult<&'a [u8], KeyMap> {
    let (in_, header) = map(
        tuple((
            mdx_number(meta),
            mdx_number(meta),
            cond(meta.is_ver2(), be_u64),
            mdx_number(meta),
            mdx_number(meta),
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
    )(in_)?;

    println!("{:?}", header);

    let (mut in_, infos) = key_block_info(in_, &header, meta)?;

    let mut keymap = KeyMap::with_capacity(header.n_entries as usize);

    for item in infos {
        let (i_, data) = content_block(in_, item.nb_compressed, item.nb_decompressed)?;
        in_ = i_;

        let (_, entries) = count(
            tuple((mdx_number(meta), mdx_string(meta))),
            item.n_entries as usize,
        )(data.as_bytes())?;

        entries.iter().for_each(|entry| {
            keymap.insert(entry.1.clone(), entry.0);
        })
    }

    Ok((in_, keymap))
}

#[derive(Debug)]
struct KeyBlockInfo {
    n_entries: u64,
    head: String,
    tail: String,
    nb_compressed: u64,
    nb_decompressed: u64,
}

type KeyMap = HashMap<String, u64>;

fn key_block_info<'a>(
    in_: &'a [u8],
    header: &KeyBlockHeader,
    meta: &DictMeta,
) -> NomResult<&'a [u8], Vec<KeyBlockInfo>> {
    fn unzip(in_: &[u8], checksum: u32) -> NomResult<&[u8], Vec<u8>> {
        nom_return!(in_, Vec<u8>, {
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

            output
        })
    }

    fn info_normal<'a>(
        in_: &'a [u8],
        header: &KeyBlockHeader,
        meta: &DictMeta,
    ) -> NomResult<&'a [u8], Vec<KeyBlockInfo>> {
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

        let (in_, infos) = count(
            map(
                tuple((
                    mdx_number(meta),
                    info_key(meta),
                    info_key(meta),
                    mdx_number(meta),
                    mdx_number(meta),
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
        )(in_)?;

        Ok((in_, infos))
    }

    let (in_, infos) = if meta.is_ver2() {
        let (in_, (_, checksum, data)) = tuple((
            le_u32,
            le_u32,
            count(le_u8, header.nb_block_info as usize - 8),
        ))(in_)?;

        let (_, input) = unzip(&data, checksum)?;

        let (_, infos) = info_normal(&input, header, meta)?;
        (in_, infos)
    } else {
        info_normal(in_, header, meta)?
    };

    // infos.iter().for_each(|info| println!("{:?}", info));

    Ok((in_, infos))
}

#[derive(Debug)]
enum ContentBlockType {
    UnCompressed = 0,
    LZO = 1,
    Zlib = 2,
}

#[derive(Debug)]
struct ContentBlock {
    block_type: ContentBlockType,
    checksum: u32,
    data: Vec<u8>,
}

fn content_block(
    in_: &[u8],
    nb_compressed: u64,
    nb_decompressed: u64,
) -> NomResult<&[u8], Vec<u8>> {
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
            count(le_u8, nb_compressed as usize),
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
                let mut output = Vec::with_capacity(nb_decompressed as usize);
                let mut decoder = ZlibDecoder::new(Cursor::new(block.data));
                decoder.read_to_end(&mut output)?;
                output
            }
            ContentBlockType::UnCompressed => block.data,
            ContentBlockType::LZO => {
                let lzo = minilzo_rs::LZO::init()?;

                lzo.decompress(&block.data, nb_decompressed as usize)?
            }
        }
    })
}

#[derive(Debug)]
struct RecordBlockHeader {
    n_blocks: u64,
    n_entries: u64,
    nb_block_info: u64,
    nb_blocks: u64,
}

#[derive(Debug)]
struct RecordBlockInfo {
    nb_compressed: u64,
    nb_decompressed: u64,
}

fn record_block<'a>(in_: &'a [u8], meta: &DictMeta) -> NomResult<&'a [u8], Record> {
    let (in_, header) = map(
        tuple((
            mdx_number(meta),
            mdx_number(meta),
            mdx_number(meta),
            mdx_number(meta),
        )),
        |(n_blocks, n_entries, nb_block_info, nb_blocks)| RecordBlockHeader {
            n_entries,
            n_blocks,
            nb_block_info,
            nb_blocks,
        },
    )(in_)?;

    let (mut in_, infos) = record_block_info(in_, &header, meta)?;
    println!("{:?}", header);

    Ok((
        in_,
        Record {
            data: infos
                .iter()
                .map(|item| {
                    // TODO: unwrap
                    let (i_, data) =
                        content_block(in_, item.nb_compressed, item.nb_decompressed).unwrap();
                    in_ = i_;
                    data
                })
                .collect::<_>(),
        },
    ))
}

fn record_block_info<'a>(
    in_: &'a [u8],
    header: &RecordBlockHeader,
    meta: &DictMeta,
) -> NomResult<&'a [u8], Vec<RecordBlockInfo>> {
    count(
        map(
            tuple((mdx_number(meta), mdx_number(meta))),
            |(nb_compressed, nb_decompressed)| RecordBlockInfo {
                nb_compressed: nb_compressed - 8,
                nb_decompressed,
            },
        ),
        header.n_blocks as usize,
    )(in_)
}

pub fn parse(in_: &[u8]) -> NomResult<&[u8], Mdx> {
    let (in_, meta) = dict_meta(in_)?;
    let (in_, keymap) = key_block(in_, &meta)?;
    let (in_, record) = record_block(in_, &meta)?;

    nom_return!(
        in_,
        Mdx,
        Mdx {
            meta,
            keymap,
            record
        }
    )
}
