def encode_popcount2(n):
    if n == 0: raise ValueError
    current = n
    blocks = []
    while current > 3:
        blocks.append(current)
        current = bin(current).count('1') - 1
    
    bits = []
    bits.append((current >> 1) & 1 == 1)
    bits.append(current & 1 == 1)
    
    for b in reversed(blocks):
        l = b.bit_length()
        for i in range(l):
            bits.append((b >> i) & 1 == 1)
    bits.append(True)
    return bits

def decode_popcount2(bits):
    i = 0
    expected_ones = 0
    if bits[i]: expected_ones += 2
    if bits[i+1]: expected_ones += 1
    i += 2
    
    current_val = expected_ones
    
    if i == len(bits) - 1 and bits[i]:
        return current_val
        
    while i < len(bits):
        if i == len(bits) - 1 and bits[i]:
            break
            
        ones_seen = 0
        block_val = 0
        bit_pos = 0
        
        while ones_seen < current_val:
            b = bits[i]
            i += 1
            if b:
                ones_seen += 1
                block_val |= (1 << bit_pos)
            bit_pos += 1
            
        while True:
            b = bits[i]
            i += 1
            if b:
                block_val |= (1 << bit_pos)
                break
            bit_pos += 1
            
        current_val = block_val
        
    return current_val

for i in range(1, 1000):
    bits = encode_popcount2(i)
    dec = decode_popcount2(bits)
    if dec != i:
        print(f"Failed for {i}: got {dec}")
print("Decode test done.")
