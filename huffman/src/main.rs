use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug)]
enum Node {
    Leaf {
        symbol: u8,
        frequency: u32,
    }, // Use u8 for ASCII
    Internal {
        frequency: u32,
        left: Box<Node>,
        right: Box<Node>,
    },
}

impl Node {
    fn frequency(&self) -> u32 {
        match self {
            Node::Leaf { frequency, .. } | Node::Internal { frequency, .. } => *frequency,
        }
    }
}

#[derive(Debug)]
struct MinNode(Box<Node>);

impl PartialEq for MinNode {
    fn eq(&self, other: &Self) -> bool {
        self.0.frequency() == other.0.frequency()
    }
}
impl Eq for MinNode {}
impl Ord for MinNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.frequency().cmp(&self.0.frequency())
    }
}
impl PartialOrd for MinNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct BitWriter {
    bits: Vec<u8>,
    current_byte: u8,
    bit_count: u8,
}

impl BitWriter {
    fn new() -> Self {
        BitWriter {
            bits: Vec::new(),
            current_byte: 0,
            bit_count: 0,
        }
    }
    fn write_bit(&mut self, bit: bool) {
        if bit {
            self.current_byte |= 1 << (7 - self.bit_count);
        }
        self.bit_count += 1;
        if self.bit_count == 8 {
            self.bits.push(self.current_byte);
            self.current_byte = 0;
            self.bit_count = 0;
        }
    }
    fn write_bits(&mut self, bits: &[bool]) {
        for &bit in bits {
            self.write_bit(bit);
        }
    }
    fn flush(&mut self) {
        if self.bit_count > 0 {
            self.bits.push(self.current_byte);
            self.bit_count = 0;
        }
    }
    fn into_bytes(self) -> Vec<u8> {
        self.bits
    }
}

struct BitReader<'a> {
    bytes: &'a [u8],
    pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        BitReader {
            bytes,
            pos: 0,
            bit_pos: 0,
        }
    }
    fn read_bit(&mut self) -> Option<bool> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let bit = (self.bytes[self.pos] & (1 << (7 - self.bit_pos))) != 0;
        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.pos += 1;
            self.bit_pos = 0;
        }
        Some(bit)
    }
}

fn build_frequency_map(input: &[u8]) -> HashMap<u8, u32> {
    let mut freq_map = HashMap::new();
    for &b in input {
        *freq_map.entry(b).or_insert(0) += 1;
    }
    freq_map
}

fn build_huffman_tree(freq_map: HashMap<u8, u32>) -> Option<Node> {
    let mut heap: BinaryHeap<MinNode> = freq_map
        .into_iter()
        .map(|(symbol, frequency)| MinNode(Box::new(Node::Leaf { symbol, frequency })))
        .collect();
    if heap.is_empty() {
        return None;
    }
    while heap.len() > 1 {
        let left = heap.pop().unwrap().0;
        let right = heap.pop().unwrap().0;
        heap.push(MinNode(Box::new(Node::Internal {
            frequency: left.frequency() + right.frequency(),
            left,
            right,
        })));
    }
    Some(*heap.pop().unwrap().0)
}

fn generate_codes(root: &Node) -> HashMap<u8, Vec<bool>> {
    let mut codes = HashMap::new();
    fn traverse(node: &Node, code: Vec<bool>, codes: &mut HashMap<u8, Vec<bool>>) {
        match node {
            Node::Leaf { symbol, .. } => {
                codes.insert(*symbol, code);
            }
            Node::Internal { left, right, .. } => {
                let mut left_code = code.clone();
                left_code.push(false);
                traverse(left, left_code, codes);
                let mut right_code = code;
                right_code.push(true);
                traverse(right, right_code, codes);
            }
        }
    }
    traverse(root, Vec::new(), &mut codes);
    codes
}

fn encode(input: &[u8], codes: &HashMap<u8, Vec<bool>>) -> Vec<u8> {
    let mut writer = BitWriter::new();
    for &b in input {
        writer.write_bits(&codes[&b]);
    }
    writer.flush();
    writer.into_bytes()
}

fn decode(encoded: &[u8], root: &Node, original_len: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let mut reader = BitReader::new(encoded);
    let mut current = root;
    while let Some(bit) = reader.read_bit() {
        current = match current {
            Node::Leaf { symbol, .. } => {
                result.push(*symbol);
                if result.len() >= original_len {
                    break;
                }
                root
            }
            Node::Internal { left, right, .. } => {
                if bit {
                    right.as_ref()
                } else {
                    left.as_ref()
                }
            }
        };
        if let Node::Leaf { symbol, .. } = current {
            result.push(*symbol);
            if result.len() >= original_len {
                break;
            }
            current = root;
        }
    }
    result
}

fn serialize_tree(root: &Node) -> Vec<u8> {
    let mut writer = BitWriter::new();
    fn serialize(node: &Node, writer: &mut BitWriter) {
        match node {
            Node::Leaf { symbol, .. } => {
                writer.write_bit(true); // 1 for leaf
                for i in (0..8).rev() {
                    writer.write_bit((symbol & (1 << i)) != 0);
                }
            }
            Node::Internal { left, right, .. } => {
                writer.write_bit(false); // 0 for internal
                serialize(left, writer);
                serialize(right, writer);
            }
        }
    }
    serialize(root, &mut writer);
    writer.flush();
    writer.into_bytes()
}

fn deserialize_tree(bytes: &[u8]) -> (Node, usize) {
    let mut reader = BitReader::new(bytes);
    fn deserialize(reader: &mut BitReader) -> Node {
        if reader.read_bit().unwrap() {
            let mut symbol = 0;
            for i in (0..8).rev() {
                if reader.read_bit().unwrap() {
                    symbol |= 1 << i;
                }
            }
            Node::Leaf {
                symbol,
                frequency: 0,
            } // Frequency not needed for decoding
        } else {
            let left = Box::new(deserialize(reader));
            let right = Box::new(deserialize(reader));
            Node::Internal {
                frequency: 0,
                left,
                right,
            }
        }
    }
    let tree = deserialize(&mut reader);
    (tree, reader.pos + (if reader.bit_pos > 0 { 1 } else { 0 }))
}

fn main() {
    println!("{:#?}", 0xC0A80101_u32.to_be_bytes());
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).unwrap(); // ~800 bytes
    let input = input.as_slice();
    println!("Original size: {} bytes", input.len());

    let freq_map = build_frequency_map(input);
    let root = build_huffman_tree(freq_map).unwrap();
    let codes = generate_codes(&root);
    let encoded = encode(input, &codes);
    let tree_data = serialize_tree(&root);
    let total_size = encoded.len() + tree_data.len() + 8; // 8 bytes for header (lengths)

    println!("Encoded size: {} bytes", encoded.len());
    println!("Tree size: {} bytes", tree_data.len());
    println!("Total size: {} bytes", total_size);

    let mut file = File::create("compressed.huf").unwrap();
    file.write_all(&(input.len() as u64).to_le_bytes()).unwrap(); // Original length
    file.write_all(&(tree_data.len() as u64).to_le_bytes())
        .unwrap(); // Tree length
    file.write_all(&tree_data).unwrap();
    file.write_all(&encoded).unwrap();

    let mut file = File::open("compressed.huf").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let original_len = u64::from_le_bytes(buffer[0..8].try_into().unwrap()) as usize;
    let tree_len = u64::from_le_bytes(buffer[8..16].try_into().unwrap()) as usize;
    let (tree, tree_bytes_read) = deserialize_tree(&buffer[16..16 + tree_len]);
    let decoded = decode(&buffer[16 + tree_bytes_read..], &tree, original_len);

    println!("Decoded size: {} bytes", decoded.len());
    assert_eq!(input, decoded.as_slice());
    println!("Success: Decompression matches original!");
}
