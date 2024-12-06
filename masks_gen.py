def generate_masks():
    def square_to_coords(square):
        """Convert square index (0-63) to (file, rank)."""
        return square % 8, square // 8

    def coords_to_square(file, rank):
        """Convert (file, rank) to square index (0-63)."""
        return rank * 8 + file

    def diagonal_mask(file, rank):
        """Calculate the diagonal mask for the given square."""
        mask = 0
        for f in range(8):
            r = rank + (f - file)
            if 0 <= r < 8 and not (f == file and r == rank):
                mask |= 1 << coords_to_square(f, r)
        return mask

    def anti_diagonal_mask(file, rank):
        """Calculate the anti-diagonal mask for the given square."""
        mask = 0
        for f in range(8):
            r = rank - (f - file)
            if 0 <= r < 8 and not (f == file and r == rank):
                mask |= 1 << coords_to_square(f, r)
        return mask

    def file_mask(file, rank):
        """Calculate the file mask for the given square."""
        mask = 0
        for r in range(8):
            if r != rank:
                mask |= 1 << coords_to_square(file, r)
        return mask

    def rank_mask(file, rank):
        """Calculate the rank mask for the given square, including the occupied bit."""
        mask = 0
        for f in range(8):
            mask |= 1 << coords_to_square(f, rank)
        return mask

    # Generate the masks for each square
    masks = []
    for square in range(64):
        file, rank = square_to_coords(square)
        diagonal_mask_ex = diagonal_mask(file, rank)
        antidiag_mask_ex = anti_diagonal_mask(file, rank)
        file_mask_ex = file_mask(file, rank)
        rank_mask_value = rank_mask(file, rank)

        masks.append({
            "diagonal_mask_ex": diagonal_mask_ex,
            "antidiag_mask_ex": antidiag_mask_ex,
            "file_mask_ex": file_mask_ex,
            "rank_mask": rank_mask_value,
        })

    return masks


def print_rust_array(masks):
    """Print the Rust array for use in the chess engine."""
    print("const SMSK: [Mask; 64] = [")
    for mask in masks:
        print(f"    Mask {{ diagonal_mask_ex: 0x{mask['diagonal_mask_ex']:016x}, "
              f"antidiag_mask_ex: 0x{mask['antidiag_mask_ex']:016x}, "
              f"file_mask_ex: 0x{mask['file_mask_ex']:016x}, "
              f"rank_mask: 0x{mask['rank_mask']:016x} }},")
    print("];")


masks = generate_masks()
print_rust_array(masks)
