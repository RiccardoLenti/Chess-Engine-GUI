def generate_knight_moves():
    knight_table = {}

    board_size = 8
    knight_offsets = [
        (2, 1), (2, -1), (-2, 1), (-2, -1),
        (1, 2), (1, -2), (-1, 2), (-1, -2)
    ]

    def is_valid_square(x, y):
        return 0 <= x < board_size and 0 <= y < board_size

    for square in range(64):
        x,y = divmod(square, board_size)

        bitboard = 0

        for dx, dy in knight_offsets:
            nx, ny = x + dx, y + dy
            if is_valid_square(nx, ny):
                dest_square = nx * board_size + ny
                bitboard |= 1 << dest_square

        knight_table[square] = bitboard

    return knight_table


knight_table = generate_knight_moves()
print("const KNIGHT_TABLE: [u64; 64] = [")
for square in range(64):
    print(f"    0x{knight_table[square]:016x},")
print("];")
