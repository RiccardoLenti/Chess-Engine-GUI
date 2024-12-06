def generate_king_moves():
    king_table = {}

    board_size = 8
    king_offsets = [
        (1, 0), (-1, 0), (0, 1), (0, -1), 
        (1, 1), (1, -1), (-1, 1), (-1, -1) 
    ]

    def is_valid_square(x, y):
        return 0 <= x < board_size and 0 <= y < board_size

    for square in range(64):
        x, y = divmod(square, board_size)

        bitboard = 0

        for dx, dy in king_offsets:
            nx, ny = x + dx, y + dy
            if is_valid_square(nx, ny):
                dest_square = nx * board_size + ny
                bitboard |= 1 << dest_square

        king_table[square] = bitboard

    return king_table


king_table = generate_king_moves()
print("const KING_TABLE: [u64; 64] = [")
for square in range(64):
    print(f"    0x{king_table[square]:016x},")
print("];")
