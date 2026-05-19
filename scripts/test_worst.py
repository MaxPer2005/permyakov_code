def omega_overhead(n):
    if n == 0: return 0
    bits = 1
    current = n
    while current > 1:
        l = current.bit_length()
        bits += l
        current = l - 1
    return bits - n.bit_length()

def popcount2_overhead(n):
    if n == 0: return 0
    raw_len = n.bit_length()
    current = n
    blocks = []
    while current > 3:
        blocks.append(current)
        current = bin(current).count('1') - 1
    bits = 2 # base is 0, 1, 2, 3
    for b in blocks: bits += b.bit_length()
    bits += 1 # terminator
    return bits - raw_len

for k in range(2, 60):
    n = (1 << k) - 1
    o = omega_overhead(n)
    p = popcount2_overhead(n)
    if p != o:
        print(f"Mismatch at k={k}: popcount={p}, omega={o}")
print("Test worst case done.")
