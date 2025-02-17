use anyhow::{anyhow, Context};
use clap::Parser;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    sync::LazyLock,
};

/*
*Value 	Char 	  	Value 	Char 	  	Value 	Char 	  	Value 	Char
0 	A 	  	16 	Q 	  	32 	g 	  	48 	w
1 	B 	  	17 	R 	  	33 	h 	  	49 	x
2 	C 	  	18 	S 	  	34 	i 	  	50 	y
3 	D 	  	19 	T 	  	35 	j 	  	51 	z
4 	E 	  	20 	U 	  	36 	k 	  	52 	0
5 	F 	  	21 	V 	  	37 	l 	  	53 	1
6 	G 	  	22 	W 	  	38 	m 	  	54 	2
7 	H 	  	23 	X 	  	39 	n 	  	55 	3
8 	I 	  	24 	Y 	  	40 	o 	  	56 	4
9 	J 	  	25 	Z 	  	41 	p 	  	57 	5
10 	K 	  	26 	a 	  	42 	q 	  	58 	6
11 	L 	  	27 	b 	  	43 	r 	  	59 	7
12 	M 	  	28 	c 	  	44 	s 	  	60 	8
13 	N 	  	29 	d 	  	45 	t 	  	61 	9
14 	O 	  	30 	e 	  	46 	u 	  	62 	+
15 	P 	  	31 	f 	  	47 	v 	  	63 	/
*
*/

#[rustfmt::skip]
static TABLE: LazyLock<HashMap<u8, char>> = LazyLock::new(|| {
    let table = [
        (0, 'A'), (1, 'B'), (2, 'C'), (3, 'D'),
        (4, 'E'), (5, 'F'), (6, 'G'), (7, 'H'),
        (8, 'I'), (9, 'J'), (10, 'K'), (11, 'L'),
        (12, 'M'), (13, 'N'), (14, 'O'), (15, 'P'),
        (16, 'Q'), (17, 'R'), (18, 'S'), (19, 'T'),
        (20, 'U'), (21, 'V'), (22, 'W'), (23, 'X'),
        (24, 'Y'), (25, 'Z'), (26, 'a'), (27, 'b'),
        (28, 'c'), (29, 'd'), (30, 'e'), (31, 'f'),
        (32, 'g'), (33, 'h'), (34, 'i'), (35, 'j'),
        (36, 'k'), (37, 'l'), (38, 'm'), (39, 'n'),
        (40, 'o'), (41, 'p'), (42, 'q'), (43, 'r'),
        (44, 's'), (45, 't'), (46, 'u'), (47, 'v'),
        (48, 'w'), (49, 'x'), (50, 'y'), (51, 'z'),
        (52, '0'), (53, '1'), (54, '2'), (55, '3'),
        (56, '4'), (57, '5'), (58, '6'), (59, '7'),
        (60, '8'), (61, '9'), (62, '+'), (63, '/'),
    ];

    table.into_iter().collect()
});

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    decode: bool,
    input: Option<String>,
}

fn get_table_value(index: u8) -> anyhow::Result<char> {
    TABLE
        .get(&index)
        .copied()
        .ok_or(anyhow!("the given base64 index is invalid"))
}

fn get_table_index(value: char) -> Option<u8> {
    TABLE
        .iter()
        .find(|(_, &map_value)| value == map_value)
        .map(|(i, _)| *i)
}

#[inline]
fn get_first_chunk(byte: u8) -> u8 {
    (byte >> 2) & 0b111111
}

#[inline]
fn get_second_chunk(byte1: u8, byte2: u8) -> u8 {
    (byte1 << 4 | byte2 >> 4) & 0b111111
}

#[inline]
fn get_third_chunk(byte2: u8, byte3: u8) -> u8 {
    (((byte2 & 0b1111) << 2) | byte3 >> 6) & 0b111111
}

#[inline]
fn get_fourth_chunk(byte3: u8) -> u8 {
    byte3 & 0b111111
}

fn encode(input: String) -> anyhow::Result<String> {
    let mut output = String::with_capacity(input.len() * 3);

    let mut input = input.as_bytes().iter();

    while let (Some(b1), b2, b3) = (input.next(), input.next(), input.next()) {
        match (b1, b2, b3) {
            (b1, Some(b2), Some(b3)) => {
                /*
                    011100 10
                    0110 0101
                    01 110011

                    011100
                    100110
                    010101
                    110011
                */

                let first_part = get_first_chunk(*b1);
                let second_part = get_second_chunk(*b1, *b2);
                let third_part = get_third_chunk(*b2, *b3);
                let fourth_part = get_fourth_chunk(*b3);

                output.push(get_table_value(first_part)?);

                output.push(get_table_value(second_part)?);
                output.push(get_table_value(third_part)?);
                output.push(get_table_value(fourth_part)?);
            }

            (b1, Some(b2), None) => {
                let first_part = get_first_chunk(*b1);
                let second_part = get_second_chunk(*b1, *b2);
                let third_part = ((b2 & 0b1111) << 2) & 0b111111;

                output.push(get_table_value(first_part)?);
                output.push(get_table_value(second_part)?);
                output.push(get_table_value(third_part)?);
                output.push('=');
            }
            (b1, None, None) => {
                let first_part = get_first_chunk(*b1);
                let second_part = (b1 << 4) & 0b111111;

                output.push(get_table_value(first_part)?);
                output.push(get_table_value(second_part)?);
                output.push_str("==");
            }

            (_, None, Some(_)) => unreachable!(),
        }
    }

    Ok(output)
}

fn decode(input: String) -> anyhow::Result<String> {
    let mut buffer = Vec::with_capacity(input.len() * 3 / 4);
    let mut input = input.chars();

    while let (Some(c1), Some(c2), Some(c3), Some(c4)) =
        (input.next(), input.next(), input.next(), input.next())
    {
        /*
            01110010
            01100101
            01110011

            011100
            10 0110
            0101 01
            110011
        */

        let not_found_error = |c: char| anyhow!("Failed to get the index by the value, value: {c}");

        let first_part_byte = get_table_index(c1).ok_or_else(|| not_found_error(c1))?;
        let second_part_byte = get_table_index(c2).ok_or_else(|| not_found_error(c2))?;
        let third_part_byte = get_table_index(c3);
        let fourth_part_byte = get_table_index(c4);

        let first_letter = first_part_byte << 2 | second_part_byte >> 4;
        buffer.push(first_letter);

        if third_part_byte.is_some() {
            let third_part_byte = third_part_byte.unwrap();

            let second_letter = ((second_part_byte & 0b1111) << 4) | third_part_byte >> 2;
            buffer.push(second_letter);

            if fourth_part_byte.is_some() {
                let fourth_part_byte = fourth_part_byte.unwrap();

                let third_letter = (third_part_byte & 0b11) << 6 | fourth_part_byte;
                buffer.push(third_letter);
            }
        }
    }

    Ok(String::from_utf8(buffer)?)
}

fn read_input(input: Option<&str>) -> anyhow::Result<String> {
    let mut buffer = String::new();

    if let Some(file) = input {
        if file == "-" {
            io::stdin().read_to_string(&mut buffer)?;
        } else {
            File::open(file)
                .with_context(|| format!("Failed to open file: {}", file))?
                .read_to_string(&mut buffer)?;
        }
    } else {
        io::stdin()
            .read_to_string(&mut buffer)
            .with_context(|| "Failed to read from the stdio".to_string())?;
    }
    Ok(buffer)
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !args.decode {
        let input = read_input(args.input.as_deref())?;
        println!("{}", encode(input)?);
    } else {
        let input = read_input(args.input.as_deref())?;
        println!("{}", decode(input)?)
    }

    Ok(())
}
