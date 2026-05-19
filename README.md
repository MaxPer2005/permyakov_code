# Permyakov Code

A highly efficient, popcount-based universal code for integer compression. It serves as a faster and often more compact alternative to standard bit-level Elias Omega codes and byte-level Varint (LEB128/VByte) encodings.

## The Algorithm

The encoding is conceptually inspired by Elias Omega but replaces the recursive bit-length encoding with **popcount** (the number of set bits in the value). This makes it extremely fast on modern CPUs thanks to hardware popcount instructions.

To solve the inherent issue of trailing zeros when decoding by popcount (because we wouldn't know when a block ends if it had trailing zeros), **the bits within each block are written in reverse order (from LSB to MSB)**. The sequence of blocks itself is written in reverse generation order. 

### Encoding Steps:
1. **Generate the sequence**: Starting with the number `N`, calculate its popcount (number of `1`s). Take that popcount and calculate *its* popcount, repeating until the value is $\le 3$.
2. **Write the Base Prefix**: The final value (1, 2, or 3) is encoded as a 2-bit prefix: `01` for 1, `10` for 2, `11` for 3.
3. **Write the Blocks**: Write all preceding blocks in reverse sequence (from smallest to largest). For each block, write its bits **from LSB to MSB**, omitting leading zeros.
4. **Terminator**: Append a final `1` bit to mark the end of the code.

### Decoding Steps:
1. Read the first 2 bits to determine how many `1`s to expect in the next block.
2. Read bits one by one until you have seen the expected number of `1`s. Because we reversed the bit order during encoding, trailing zeros (which are now leading zeros in the reversed bitstream) are naturally handled: we just keep reading until we hit the required number of set bits.
3. The value we just read becomes the required number of `1`s for the *next* block.
4. Repeat until you hit the terminator bit `1`.

## Theoretical Dominance

**Permyakov Code strictly dominates Elias Omega: its structural overhead is always $\le$ Omega (with the sole exception of $N=1$), providing optimal compression across the integer domain.**

The fundamental property of the Permyakov Code is that its metadata overhead is driven by the Hamming weight (`popcount`) rather than the bit-length (`log2`). By subtracting $1$ from the popcount at each level of the chain (since the MSB of any block is always 1 and can be inferred during reversed decoding), the Permyakov Code achieves an even faster chain collapse. 

For any integer $N$, the following strict inequality holds:

$$ \text{popcount}(N) - 1 < \lfloor\log_2 N\rfloor $$

Because the modified popcount sequence converges strictly faster than the bit-length sequence used by Elias Omega for all integers, the structural overhead is completely minimized. In the absolute worst-case scenario ($N = 2^k - 1$, where all bits are `1`), the Permyakov Code overhead matches Elias Omega exactly. For the vast majority of integers, the Permyakov Code is drastically smaller.

### Best Case vs Worst Case Overhead
The charts below visualize the structural overhead (metadata bits excluding the raw binary value of $N$) as the number grows up to $2^{60}$.

**1. Best Case ($N = 2^k$):** The number contains a single `1` bit.
![Best Case](assets/best_case.svg)
*While Elias Omega overhead grows logarithmically, Permyakov Code overhead remains perfectly constant at 3 bits.*

**2. Worst Case ($N = 2^k - 1$):** All bits are `1`.
![Worst Case](assets/worst_case.svg)
*Even in its absolute worst-case scenario, the Permyakov Code overhead never exceeds Elias Omega.*

---

## Benchmarks

### 1. Permyakov Codes vs Elias Omega (General Distribution)
When encoding numbers from 1 to 1,000,000, Permyakov Codes provide significant savings in structural overhead compared to standard Elias Omega.

| Metric | Permyakov Code | Elias Omega |
|--------|---------------|-------------|
| Total Overhead | **6,896,424 bits** | 10,737,553 bits |
| Size | **64.2%** of Omega | 100% |

**Advantage:** Permyakov meta-data overhead is ~36% smaller than Elias Omega, while being significantly faster to encode/decode due to hardware `popcount`.

### 2. Permyakov Codes vs Varint (VByte) in Inverted Indexes
In a real-world scenario like an inverted index (where we encode *deltas* between sorted IDs), we require a log-based index to achieve $O(\log K)$ Random Access.

We benchmarked a dense array (1 Million elements, average delta ~2) where both the data and a recursive Skip-List/B-Tree index are compressed. The benchmark measures both the memory footprint and the speed of executing 10,000 Random Access (`select`) queries at different skip factors (`G`).

| Skip G | Permyakov Size | Query Time (10k) | Varint Size | Query Time (10k) | Advantage |
|--------|----------------|------------------|-------------|------------------|-----------|
| 4      | 14.15 Mbits    | 6.55 ms          | 15.91 Mbits | 1.35 ms          | 1.12x smaller |
| 8      | 8.45 Mbits     | 7.06 ms          | 11.39 Mbits | 1.07 ms          | 1.35x smaller |
| **16** | **6.18 Mbits** | **10.14 ms**     | **9.58 Mbits**| **1.19 ms**      | **1.55x smaller** |
| 32     | 5.15 Mbits     | 13.23 ms         | 8.76 Mbits  | 1.32 ms          | 1.70x smaller |
| 64     | 4.67 Mbits     | 19.48 ms         | 8.37 Mbits  | 1.91 ms          | 1.79x smaller |
| 128    | 4.43 Mbits     | 29.39 ms         | 8.18 Mbits  | 2.40 ms          | 1.85x smaller |

## Adaptive Permutation of Sorted Runs

To further optimize decoding speed within the logarithmic index blocks, the Permyakov Code index structure supports **Adaptive Permutation of Sorted Runs**.

In typical search indexes, query frequencies follow a Zipf distribution (e.g., 80% of queries hit 20% of elements). By default, finding a "hot" element requires scanning half of the block on average. To fix this without breaking the decoder's ability to locate original logic indices, we adaptively partition the block into up to 4 **Sorted Runs**. 

These continuous sub-arrays of the original block are then permuted so that the run containing the hottest elements is moved to the front. Because the runs are strictly ordered in decreasing magnitude of their original positions, the decoder can deduce the original logical index mathematically without needing explicit bit-masks.

**Overhead:**
- `runs_count` (1 to 4) = 2 bits
- Split positions = up to 3 * $\lceil\log_2(G)\rceil$ bits.
- Total overhead for $G=16$: max 14 bits per block (less than 1 bit per element).

**Benchmark (G=16, Zipf 80/20):**
| Metric | Average Reads per Query |
|--------|-------------------------|
| Baseline (No Permutation) | 8.53 reads |
| Adaptive Permutation | **4.75 reads** |
| **Speedup** | **1.80x fewer reads** |

This structural optimization brings the Random Access speed of the Permyakov Code significantly closer to byte-aligned formats like Varint, while maintaining its massive memory advantage.

---

## Running the Benchmarks

This repository contains a Rust MVP demonstrating the algorithm and the benchmarks.

```bash
cargo run --release
```

## License
MIT