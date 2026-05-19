import math

def omega_overhead(n):
    if n == 0: return 0
    bits = 1
    current = n
    raw_len = n.bit_length()
    while current > 1:
        l = current.bit_length()
        bits += l
        current = l - 1
    return bits - raw_len

def popcount_overhead(n):
    if n == 0: return 0
    raw_len = n.bit_length()
    current = n
    blocks = []
    while current > 3:
        blocks.append(current)
        if hasattr(int, 'bit_count'):
            current = current.bit_count() - 1
        else:
            current = bin(current).count('1') - 1
            
    bits = 2 # base is 0, 1, 2, 3
    for b in blocks:
        bits += b.bit_length()
    bits += 1 # terminator
    return bits - raw_len

def generate_svg(filename, title, data_omega, data_permyakov, x_labels, max_y):
    width = 600
    height = 350
    margin_left = 60
    margin_right = 20
    margin_top = 50
    margin_bottom = 50
    
    graph_width = width - margin_left - margin_right
    graph_height = height - margin_top - margin_bottom
    
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}">\n'
    svg += f'  <rect width="{width}" height="{height}" fill="#ffffff"/>\n'
    svg += f'  <text x="{width/2}" y="30" font-family="Arial" font-size="16" font-weight="bold" text-anchor="middle">{title}</text>\n'
    
    # Axes
    svg += f'  <line x1="{margin_left}" y1="{height-margin_bottom}" x2="{width-margin_right}" y2="{height-margin_bottom}" stroke="#000" stroke-width="1"/>\n'
    svg += f'  <line x1="{margin_left}" y1="{margin_top}" x2="{margin_left}" y2="{height-margin_bottom}" stroke="#000" stroke-width="1"/>\n'
    
    # Y-axis labels
    y_steps = 5
    for i in range(y_steps + 1):
        y_val = int(max_y * i / y_steps)
        y_pos = height - margin_bottom - (y_val / max_y * graph_height)
        svg += f'  <text x="{margin_left-10}" y="{y_pos+4}" font-family="Arial" font-size="12" text-anchor="end">{y_val} bits</text>\n'
        svg += f'  <line x1="{margin_left}" y1="{y_pos}" x2="{width-margin_right}" y2="{y_pos}" stroke="#e0e0e0" stroke-width="1"/>\n'
        
    # X-axis labels
    x_steps = len(x_labels)
    for i, label in enumerate(x_labels):
        if i % 5 == 0 or i == x_steps - 1:
            x_pos = margin_left + (i / (x_steps - 1) * graph_width)
            svg += f'  <text x="{x_pos}" y="{height-margin_bottom+20}" font-family="Arial" font-size="12" text-anchor="middle">{label}</text>\n'
            svg += f'  <line x1="{x_pos}" y1="{height-margin_bottom}" x2="{x_pos}" y2="{height-margin_bottom+5}" stroke="#000" stroke-width="1"/>\n'
    svg += f'  <text x="{width/2}" y="{height-10}" font-family="Arial" font-size="14" text-anchor="middle">Log2(N) (Length of Value)</text>\n'

    # Legend
    svg += f'  <rect x="{margin_left+20}" y="{margin_top+10}" width="15" height="15" fill="#ff7f0e"/>\n'
    svg += f'  <text x="{margin_left+40}" y="{margin_top+22}" font-family="Arial" font-size="12">Elias Omega Overhead</text>\n'
    svg += f'  <rect x="{margin_left+20}" y="{margin_top+30}" width="15" height="15" fill="#1f77b4"/>\n'
    svg += f'  <text x="{margin_left+40}" y="{margin_top+42}" font-family="Arial" font-size="12">Permyakov Code Overhead</text>\n'

    # Lines
    def get_points(data):
        pts = []
        for i, val in enumerate(data):
            x = margin_left + (i / (x_steps - 1) * graph_width)
            y = height - margin_bottom - (val / max_y * graph_height)
            pts.append(f"{x},{y}")
        return " ".join(pts)
        
    svg += f'  <polyline points="{get_points(data_omega)}" fill="none" stroke="#ff7f0e" stroke-width="3" stroke-linejoin="round"/>\n'
    svg += f'  <polyline points="{get_points(data_permyakov)}" fill="none" stroke="#1f77b4" stroke-width="3" stroke-linejoin="round"/>\n'
    
    svg += '</svg>\n'
    with open(filename, 'w') as f:
        f.write(svg)

# Data generation
k_values = list(range(2, 65))
x_labels = [str(k) for k in k_values]

# Best Case: N = 2^k (Popcount = 1)
omega_best = [omega_overhead(2**k) for k in k_values]
perm_best = [popcount_overhead(2**k) for k in k_values]
generate_svg("best_case.svg", "Best Case (Popcount = 1): N = 2^k", omega_best, perm_best, x_labels, max(omega_best)+2)

# Worst Case: N = 2^k - 1 (Popcount = k)
omega_worst = [omega_overhead(2**k - 1) for k in k_values]
perm_worst = [popcount_overhead(2**k - 1) for k in k_values]
generate_svg("worst_case.svg", "Worst Case (Popcount = k): N = 2^k - 1", omega_worst, perm_worst, x_labels, max(omega_worst)+2)

print("SVGs generated.")
