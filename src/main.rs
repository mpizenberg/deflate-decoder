// use nom::bytes::complete::{tag, take, take_till};
use nom::combinator::{map, map_res};
// use nom::multi::many1;
// use nom::number::complete::{be_u16, be_u32, be_u8};
use nom::bits::bits;
use nom::error::{ErrorKind, ParseError};
use nom::sequence::tuple;
use nom::{Err, IResult};
use nom::{InputIter, InputLength, Slice, ToUsize};
use std::ops::{AddAssign, Div, RangeFrom, Shl, Shr};
use std::{env, error::Error, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(error) = run(&args) {
        eprintln!("{:?}", error);
    }
}

fn run(args: &[String]) -> Result<(), Box<Error>> {
    // let data = fs::read(&args[1])?;
    match parse_block_header(&HELLO[0..]) {
        Ok((_, n)) => println!("{:?}", n),
        Err(e) => eprintln!("{:?}", e),
    };
    println!("All done!");
    Ok(())
}

type ByteInput<'a> = (&'a [u8], usize);

const HELLO: &[u8] = &[243, 72, 205, 201, 201, 215, 81, 40, 207, 47, 202, 73, 1, 0];
const ONE: &[u8] = &[1, 128];

#[derive(Debug)]
struct Block {
    b_final: bool,
    b_type: BlockType,
    data: usize,
}

#[derive(Debug)]
enum BlockType {
    Raw,
    StaticHuffman,
    DynamicHuffman,
    Reserved,
}

fn parse_block(input: &[u8]) -> IResult<&[u8], Block> {
    let b_final = true;
    let b_type = BlockType::Raw;
    let data = 42;
    Ok((
        input,
        Block {
            b_final,
            b_type,
            data,
        },
    ))
}

fn parse_block_header(input: &[u8]) -> IResult<&[u8], (u8, u8, u8)> {
    bits::<_, _, (_, _), _, _>(tuple((
        take_increase(1_usize),
        take_increase(2_usize),
        take_increase(5_usize),
    )))(input)
}

// --------------------- Parsing helper ----------------------------

pub fn take_increase<I, O, C, E: ParseError<(I, usize)>>(
    count: C,
) -> impl Fn((I, usize)) -> IResult<(I, usize), O, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    C: ToUsize,
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
{
    let count = count.to_usize();
    move |(input, bit_offset): (I, usize)| {
        if count == 0 {
            Ok(((input, bit_offset), 0u8.into()))
        } else {
            let cnt = (count + bit_offset).div(8);
            if input.input_len() * 8 < count + bit_offset {
                Err(Err::Error(E::from_error_kind(
                    (input, bit_offset),
                    ErrorKind::Eof,
                )))
            } else {
                let mut acc: O = (0 as u8).into();
                let mut acc_nb: usize = 0;
                let mut offset: usize = bit_offset;
                let mut remaining: usize = count;
                let mut end_offset: usize = 0;

                for byte in input.iter_elements().take(cnt + 1) {
                    if remaining == 0 {
                        break;
                    }

                    // overflow = negative underflow
                    let underflow: isize = 8 - offset as isize - remaining as isize;

                    // val contains our bits in this byte,
                    // with higher significance bits set to 0 (left padded)
                    let val: O = if underflow < 0 {
                        byte.into()
                    } else {
                        ((byte << underflow) >> underflow).into()
                    };

                    if offset > 0 {
                        // necessarily the beginning of bit retrieving
                        acc = val >> offset;
                    } else {
                        // the val has higher weight (more significant bits)
                        acc += val << acc_nb;
                    }

                    if underflow > 0 {
                        // we just need to update end_offset if no overflow
                        end_offset = remaining + offset;
                        break;
                    } else {
                        acc_nb += 8 - offset;
                        remaining -= 8 - offset;
                        offset = 0;
                    }
                }
                Ok(((input.slice(cnt..), end_offset), acc))
            }
        }
    }
}
