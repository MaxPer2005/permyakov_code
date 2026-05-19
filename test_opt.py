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
    bits = 2 # base is 0, 1, 2, 3 (2 bits)
    for b in blocks: bits += b.bit_length()
    bits += 1 # terminator
    return bits - raw_len

losses = []
for i in range(1, 100001):
    if popcount2_overhead(i) > omega_overhead(i):
        losses.append(i)

print(f"Losses count: {len(losses)}")
if len(losses) > 0:
    print(f"Losses: {losses[:20]}")
