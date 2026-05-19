fn encode_permyakov(n: u64) -> Vec<bool> {
    if n == 0 {
        panic!("Only positive integers are supported");
    }
    let mut current = n;
    let mut blocks = Vec::new();
    while current > 3 {
        blocks.push(current);
        current = current.count_ones() as u64;
    }
    let mut bits = Vec::new();
    bits.push((current >> 1) & 1 == 1);
    bits.push((current & 1) == 1);
    for &block in blocks.iter().rev() {
        let len = 64 - block.leading_zeros();
        for i in 0..len {
            bits.push((block >> i) & 1 == 1);
        }
    }
    bits.push(true);
    bits
}

fn varint_bytes(n: u64) -> u64 {
    let mut size = 0;
    let mut current = n;
    loop {
        size += 1;
        current >>= 7;
        if current == 0 { break; }
    }
    size
}

fn run_benchmark(deltas: &[u64], benchmark_name: &str) {
    println!("=== {} ===", benchmark_name);
    let mut total_permyakov_bits: u64 = 0;
    let mut total_varint_bits: u64 = 0;
    
    let mut permyakov_sizes = Vec::with_capacity(deltas.len());
    let mut varint_sizes_bytes = Vec::with_capacity(deltas.len());
    
    for &d in deltas {
        // Delta can be 0 if there are duplicates. 
        // Standard practice in inverted indexes is to encode delta + 1.
        let val = d + 1;
        
        let p_len = encode_permyakov(val).len() as u64;
        permyakov_sizes.push(p_len);
        total_permyakov_bits += p_len;
        
        let v_bytes = varint_bytes(val);
        varint_sizes_bytes.push(v_bytes);
        total_varint_bits += v_bytes * 8;
    }
    
    println!("  Permyakov Data: {:>10} bits", total_permyakov_bits);
    println!("  Varint Data  : {:>10} bits", total_varint_bits);
    
    let n_group = 8;
    
    // --- Permyakov Index ---
    let mut permyakov_index_bits = 0;
    let mut p_offsets = Vec::new();
    let mut current_p_offset = 0;
    for &sz in &permyakov_sizes {
        p_offsets.push(current_p_offset);
        current_p_offset += sz;
    }
    
    loop {
        let mut next_level_data = Vec::new();
        for (i, &off) in p_offsets.iter().enumerate() {
            if i % n_group == 0 { next_level_data.push(off); }
        }
        if next_level_data.len() <= 1 { break; }
        
        let mut level_bits = 0;
        let mut next_p_offsets = Vec::new();
        let mut off = 0;
        for &val in &next_level_data {
            let encoded_len = encode_permyakov(val + 1).len() as u64;
            level_bits += encoded_len;
            next_p_offsets.push(off);
            off += encoded_len;
        }
        permyakov_index_bits += level_bits;
        p_offsets = next_p_offsets;
    }
    
    // --- Varint Index ---
    let mut varint_index_bits = 0;
    let mut v_offsets = Vec::new();
    let mut current_v_offset = 0;
    for &sz in &varint_sizes_bytes {
        v_offsets.push(current_v_offset);
        current_v_offset += sz;
    }
    
    loop {
        let mut next_level_data = Vec::new();
        for (i, &off) in v_offsets.iter().enumerate() {
            if i % n_group == 0 { next_level_data.push(off); }
        }
        if next_level_data.len() <= 1 { break; }
        
        let mut level_bytes = 0;
        let mut next_v_offsets = Vec::new();
        let mut off = 0;
        for &val in &next_level_data {
            let encoded_bytes = varint_bytes(val + 1);
            level_bytes += encoded_bytes;
            next_v_offsets.push(off);
            off += encoded_bytes;
        }
        varint_index_bits += level_bytes * 8;
        v_offsets = next_v_offsets;
    }
    
    println!("  Permyakov Index Overhead : {:>10} bits", permyakov_index_bits);
    println!("  Varint Index Overhead   : {:>10} bits", varint_index_bits);
    
    let total_permyakov = total_permyakov_bits + permyakov_index_bits;
    let total_varint = total_varint_bits + varint_index_bits;
    
    println!("--------------------------------------------------");
    println!("  TOTAL Permyakov (Data+Idx) : {:>10} bits", total_permyakov);
    println!("  TOTAL Varint   (Data+Idx) : {:>10} bits", total_varint);
    println!("--------------------------------------------------");
    
    if total_varint > total_permyakov {
        let diff = total_varint - total_permyakov;
        let ratio = total_varint as f64 / total_permyakov as f64;
        println!("  🏆 Permyakov is {:.2}x SMALLER! (Saves {} bits)", ratio, diff);
    } else {
        let diff = total_permyakov - total_varint;
        let ratio = total_permyakov as f64 / total_varint as f64;
        println!("  ⚖️ Varint is {:.2}x smaller (Saves {} bits)", ratio, diff);
    }
    println!("\n");
}

fn main() {
    let n = 1_000_000;
    
    // Simple LCG PRNG
    struct Rng { state: u64 }
    impl Rng {
        fn next_u32(&mut self) -> u32 {
            self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (self.state >> 32) as u32
        }
    }
    let mut rng = Rng { state: 12345 };
    
    println!("=== DELTA-ENCODING BENCHMARK (1M Sorted Random Elements) ===\n");
    
    // Scenario 1: DENSE ARRAY (Like Search Engine inverted indexes)
    // Values range from 0 to 2,000,000. Average delta is ~2.
    let mut dense_arr = Vec::with_capacity(n);
    for _ in 0..n { dense_arr.push(rng.next_u32() % 2_000_000); }
    dense_arr.sort_unstable();
    
    let mut dense_deltas = Vec::with_capacity(n);
    dense_deltas.push(dense_arr[0] as u64);
    for i in 1..n { dense_deltas.push((dense_arr[i] - dense_arr[i-1]) as u64); }
    
    run_benchmark(&dense_deltas, "Scenario 1: DENSE DATA (Avg Delta ~ 2)");

    // Scenario 2: SPARSE ARRAY (Full u32 domain)
    // Values range from 0 to u32::MAX. Average delta is ~4294.
    let mut sparse_arr = Vec::with_capacity(n);
    for _ in 0..n { sparse_arr.push(rng.next_u32()); }
    sparse_arr.sort_unstable();
    
    let mut sparse_deltas = Vec::with_capacity(n);
    sparse_deltas.push(sparse_arr[0] as u64);
    for i in 1..n { sparse_deltas.push((sparse_arr[i] - sparse_arr[i-1]) as u64); }
    
    run_benchmark(&sparse_deltas, "Scenario 2: SPARSE DATA (Avg Delta ~ 4294)");
}