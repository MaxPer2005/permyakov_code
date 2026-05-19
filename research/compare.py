def omega_overhead(n):
    if n == 0: return 0
    bits = 1
    current = n
    while current > 1:
        l = current.bit_length()
        bits += l
        current = l - 1
    return bits - n.bit_length()

def popcount_overhead(n):
    if n == 0: return 0
    current = n
    blocks = []
    while current > 3:
        blocks.append(current)
        if hasattr(int, 'bit_count'): current = current.bit_count()
        else: current = bin(current).count('1')
    bits = 3
    for b in blocks: bits += b.bit_length()
    return bits - n.bit_length()

for i in range(1, 1000):
    if popcount_overhead(i) > omega_overhead(i):
        print(f"Loss at N={i} (bin: {bin(i)}), Popcount overhead: {popcount_overhead(i)}, Omega: {omega_overhead(i)}")
