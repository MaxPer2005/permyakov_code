use std::time::Instant;

// --- Permyakov Code (Bit-level) ---

struct BitWriter {
    data: Vec<u64>,
    bit_len: usize,
}

impl BitWriter {
    fn new() -> Self {
        Self { data: Vec::new(), bit_len: 0 }
    }
    
    fn push(&mut self, bit: bool) {
        let idx = self.bit_len / 64;
        if idx >= self.data.len() {
            self.data.push(0);
        }
        if bit {
            self.data[idx] |= 1 << (self.bit_len % 64);
        }
        self.bit_len += 1;
    }
}

struct BitReader<'a> {
    data: &'a [u64],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u64], bit_pos: usize) -> Self {
        Self { data, bit_pos }
    }
    
    fn read(&mut self) -> bool {
        let bit = (self.data[self.bit_pos / 64] >> (self.bit_pos % 64)) & 1 == 1;
        self.bit_pos += 1;
        bit
    }
}

fn encode_prefix(len: usize, base: u64, writer: &mut BitWriter) {
    match len {
        0 => { writer.push(false); }
        1 => { writer.push(true); writer.push(false); }
        2 => { writer.push(true); writer.push(true); writer.push(false); }
        _ => { writer.push(true); writer.push(true); writer.push(true); }
    }
    writer.push((base >> 1) & 1 == 1);
    writer.push((base & 1) == 1);
}

fn decode_prefix(reader: &mut BitReader) -> (usize, u64) {
    let len = if !reader.read() { 0 }
              else if !reader.read() { 1 }
              else if !reader.read() { 2 }
              else { 3 };
    let mut base = 0;
    if reader.read() { base |= 2; }
    if reader.read() { base |= 1; }
    (len, base)
}

fn encode_permyakov(n: u64, writer: &mut BitWriter) {
    if n == 0 { panic!("Only positive integers supported"); }
    let mut current = n;
    let mut blocks = Vec::new();
    while current > 3 {
        blocks.push(current);
        current = (current.count_ones() as u64) - 1;
    }
    encode_prefix(blocks.len(), current, writer);
    
    for &block in blocks.iter().rev() {
        let len = 64 - block.leading_zeros();
        for i in 0..len {
            writer.push((block >> i) & 1 == 1);
        }
    }
}

fn decode_permyakov(reader: &mut BitReader) -> u64 {
    let (len, base) = decode_prefix(reader);
    let mut current_val = base;
    
    for _ in 0..len {
        let mut ones_seen = 0;
        let mut block_val = 0;
        let mut bit_pos = 0;
        
        while ones_seen < current_val {
            if reader.read() {
                ones_seen += 1;
                block_val |= 1 << bit_pos;
            }
            bit_pos += 1;
        }
        
        loop {
            if reader.read() {
                block_val |= 1 << bit_pos;
                break;
            }
            bit_pos += 1;
        }
        
        current_val = block_val;
    }
    current_val
}

struct BitStream {
    data: Vec<u64>,
    bit_len: usize,
}

struct DatabasePermyakov {
    data_stream: BitStream,
    index_levels: Vec<BitStream>,
    n_group: usize,
}

impl DatabasePermyakov {
    fn build(deltas: &[u64], n_group: usize) -> Self {
        let mut data_writer = BitWriter::new();
        let mut offsets = Vec::new();
        
        for &d in deltas {
            offsets.push(data_writer.bit_len as u64);
            encode_permyakov(d + 1, &mut data_writer);
        }
        
        let mut index_levels = Vec::new();
        let mut current_offsets = offsets;
        
        loop {
            let mut next_level_data = Vec::new();
            for (i, &off) in current_offsets.iter().enumerate() {
                if i % n_group == 0 {
                    next_level_data.push(off);
                }
            }
            if next_level_data.len() <= 1 { break; }
            
            let mut level_writer = BitWriter::new();
            let mut next_offsets = Vec::new();
            for &val in &next_level_data {
                next_offsets.push(level_writer.bit_len as u64);
                encode_permyakov(val + 1, &mut level_writer);
            }
            
            index_levels.push(BitStream { data: level_writer.data, bit_len: level_writer.bit_len });
            current_offsets = next_offsets;
        }
        
        Self {
            data_stream: BitStream { data: data_writer.data, bit_len: data_writer.bit_len },
            index_levels,
            n_group,
        }
    }
    
    fn select(&self, target_idx: usize) -> u64 {
        if self.index_levels.is_empty() {
            let mut reader = BitReader::new(&self.data_stream.data, 0);
            let mut val = 0;
            for _ in 0..=target_idx {
                val = decode_permyakov(&mut reader);
            }
            return val - 1;
        }
        
        let mut level_idx = self.index_levels.len();
        let mut bit_offset = 0;
        
        while level_idx > 0 {
            level_idx -= 1;
            let current_level_group_size = self.n_group.pow((level_idx + 1) as u32);
            let target_in_this_level = target_idx / current_level_group_size;
            
            let stream = &self.index_levels[level_idx];
            let mut reader = BitReader::new(&stream.data, bit_offset);
            
            let skips = target_in_this_level % self.n_group;
            let mut val = 0;
            for _ in 0..=skips {
                val = decode_permyakov(&mut reader);
            }
            
            bit_offset = (val - 1) as usize;
        }
        
        let mut reader = BitReader::new(&self.data_stream.data, bit_offset);
        let skips = target_idx % self.n_group;
        let mut val = 0;
        for _ in 0..=skips {
            val = decode_permyakov(&mut reader);
        }
        val - 1
    }
}

// --- Varint (VByte) ---

fn encode_varint(mut n: u64, out: &mut Vec<u8>) {
    loop {
        let mut b = (n & 0x7f) as u8;
        n >>= 7;
        if n != 0 {
            b |= 0x80;
            out.push(b);
        } else {
            out.push(b);
            break;
        }
    }
}

fn decode_varint(data: &[u8], pos: &mut usize) -> u64 {
    let mut res = 0u64;
    let mut shift = 0;
    loop {
        let b = data[*pos];
        *pos += 1;
        res |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    res
}

struct DatabaseVarint {
    data_stream: Vec<u8>,
    index_levels: Vec<Vec<u8>>,
    n_group: usize,
}

impl DatabaseVarint {
    fn build(deltas: &[u64], n_group: usize) -> Self {
        let mut data_stream = Vec::new();
        let mut offsets = Vec::new();
        
        for &d in deltas {
            offsets.push(data_stream.len() as u64);
            encode_varint(d + 1, &mut data_stream);
        }
        
        let mut index_levels = Vec::new();
        let mut current_offsets = offsets;
        
        loop {
            let mut next_level_data = Vec::new();
            for (i, &off) in current_offsets.iter().enumerate() {
                if i % n_group == 0 {
                    next_level_data.push(off);
                }
            }
            if next_level_data.len() <= 1 { break; }
            
            let mut level_stream = Vec::new();
            let mut next_offsets = Vec::new();
            for &val in &next_level_data {
                next_offsets.push(level_stream.len() as u64);
                encode_varint(val + 1, &mut level_stream); // encode byte offset
            }
            
            index_levels.push(level_stream);
            current_offsets = next_offsets;
        }
        
        Self {
            data_stream,
            index_levels,
            n_group,
        }
    }
    
    fn select(&self, target_idx: usize) -> u64 {
        if self.index_levels.is_empty() {
            let mut pos = 0;
            let mut val = 0;
            for _ in 0..=target_idx {
                val = decode_varint(&self.data_stream, &mut pos);
            }
            return val - 1;
        }
        
        let mut level_idx = self.index_levels.len();
        let mut byte_offset = 0;
        
        while level_idx > 0 {
            level_idx -= 1;
            let current_level_group_size = self.n_group.pow((level_idx + 1) as u32);
            let target_in_this_level = target_idx / current_level_group_size;
            
            let stream = &self.index_levels[level_idx];
            let mut pos = byte_offset;
            
            let skips = target_in_this_level % self.n_group;
            let mut val = 0;
            for _ in 0..=skips {
                val = decode_varint(stream, &mut pos);
            }
            
            byte_offset = (val - 1) as usize;
        }
        
        let mut pos = byte_offset;
        let skips = target_idx % self.n_group;
        let mut val = 0;
        for _ in 0..=skips {
            val = decode_varint(&self.data_stream, &mut pos);
        }
        val - 1
    }
}


struct Rng { state: u64 }
impl Rng {
    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }
}

fn main() {
    let n = 1_000_000;
    let mut rng = Rng { state: 12345 };
    
    let mut arr = Vec::with_capacity(n);
    for _ in 0..n { arr.push(rng.next_u32() % 2_000_000); }
    arr.sort_unstable();
    
    let mut deltas = Vec::with_capacity(n);
    deltas.push(arr[0] as u64);
    for i in 1..n { deltas.push((arr[i] - arr[i-1]) as u64); }
    
    let groups = [4, 8, 16, 32, 64, 128, 256];
    let num_queries = 10_000;
    
    println!("=== B-Tree Index Navigation Benchmark (Permyakov vs Varint) ===");
    println!("Data: 1,000,000 dense deltas (avg ~2)\n");
    println!("{:<6} | {:<25} | {:<25}", "Skip G", "Permyakov Code", "Varint (VByte)");
    println!("{:<6} | {:<12} | {:<10} | {:<12} | {:<10}", "", "Size (bits)", "Time(10k)", "Size (bits)", "Time(10k)");
    println!("-----------------------------------------------------------------------");
    
    for &g in &groups {
        let db_p = DatabasePermyakov::build(&deltas, g);
        let mut p_index_bits = 0;
        for lvl in &db_p.index_levels { p_index_bits += lvl.bit_len; }
        let p_total_bits = db_p.data_stream.bit_len + p_index_bits;
        
        let mut query_rng = Rng { state: 777 };
        let start_p = Instant::now();
        let mut chk_p = 0;
        for _ in 0..num_queries {
            let idx = (query_rng.next_u32() as usize) % n;
            chk_p ^= db_p.select(idx);
        }
        let t_p = start_p.elapsed();
        
        let db_v = DatabaseVarint::build(&deltas, g);
        let mut v_index_bytes = 0;
        for lvl in &db_v.index_levels { v_index_bytes += lvl.len(); }
        let v_total_bits = (db_v.data_stream.len() + v_index_bytes) * 8;
        
        let mut query_rng2 = Rng { state: 777 };
        let start_v = Instant::now();
        let mut chk_v = 0;
        for _ in 0..num_queries {
            let idx = (query_rng2.next_u32() as usize) % n;
            chk_v ^= db_v.select(idx);
        }
        let t_v = start_v.elapsed();
        
        assert_eq!(chk_p, chk_v, "Checksum mismatch!");
        
        let size_ratio = v_total_bits as f64 / p_total_bits as f64;
        let p_time_str = format!("{:?}", t_p);
        let v_time_str = format!("{:?}", t_v);
        
        println!("{:<6} | {:<12} | {:<10} | {:<12} | {:<10} (P is {:.2}x smaller)", 
                 g, p_total_bits, p_time_str, v_total_bits, v_time_str, size_ratio);
    }
}