#!/bin/env python3
import os
import sys
from collections import Counter

# ANSI color codes

RED = "\033[91m"
GREEN = "\033[92m"
ORANGE = "\033[93m"
BLUE = "\033[94m"
MAGENTA = "\033[95m"
CYAN = "\033[96m"
RESET = "\033[0m"

EXT2COLOR = {
    "jxl": CYAN,
    "png": MAGENTA,
    "jpg": RED,
    "jpeg": RED,
    "webp": MAGENTA,
    "caption": BLUE,
    "txt": BLUE,
}
EXT2ORDER = {ext: i for i, ext in enumerate(EXT2COLOR.keys())}
SORT_KEYS = ["name", "count", "image", "text", *EXT2COLOR.keys()]

TEXT_FORMATS = {"txt", "caption"}
IMAGE_FORMATS = EXT2COLOR.keys() - TEXT_FORMATS


def count_files(directory):
    file_counts = Counter()
    for root, dirs, files in os.walk(directory):
        for file in files:
            base_name, ext = os.path.splitext(file)
            if len(ext) > 1:
                ext = ext[1:]
                file_counts[ext] += 1
                if ext in IMAGE_FORMATS:
                    file_counts["image"] += 1
                elif ext in TEXT_FORMATS:
                    file_counts["text"] += 1

    return file_counts


def main():
    sort_key_name = "name"
    sort_reverse = False
    if len(sys.argv) > 1:
        sort_key_name = sys.argv[1]
    if sort_key_name.endswith("_r"):
        sort_reverse = True
        sort_key_name = sort_key_name[:-2]

    if sort_key_name == "name":
        sort_key = lambda x: x[0]
    elif sort_key_name == "count":
        sort_key = lambda x: x[1]
    elif sort_key_name in SORT_KEYS:
        sort_key = lambda x: x[2].get(sort_key_name, 0)
    else:
        print(f'Valid short key are {", ".join(f'"{k}"' for k in SORT_KEYS)}')
        print('Prepending "_r" to reverse the sort order')
        sys.exit(1)

    current_directory = os.getcwd()
    directories = (
        d
        for d in os.listdir(current_directory)
        if os.path.isdir(os.path.join(current_directory, d))
    )

    stats = []
    grand_total = Counter()
    for directory in directories:
        dir_path = os.path.join(current_directory, directory)
        counts = count_files(dir_path)
        total_files = sum(v for k,v in counts.items() if k in EXT2ORDER)
        stats.append((directory, total_files, counts))
        grand_total.update(counts)

    stats.sort(key=sort_key, reverse=sort_reverse)
    stats.append((None, sum(v for k,v in grand_total.items() if k in EXT2ORDER), grand_total))

    for directory, total_files, counts in stats:
        if total_files == 0:
            continue
        if directory is None:
            print(f'Grand Total: ')
        print(f"Directory: {directory}")
        for ext, count in sorted(
            counts.items(), key=lambda x: EXT2ORDER.get(x[0], -1)
        ):
            if counts[ext] == 0 or ext not in EXT2COLOR:
                continue
            print(f"{EXT2COLOR[ext]}{ext} files: {counts[ext]}{RESET}")
        tally_color = GREEN if total_files >= 200 else ORANGE
        print(
            f"{tally_color}Total files: {total_files}{RESET} ({counts['image']} images, {counts['text']} texts)"
        )
        print()


if __name__ == "__main__":
    main()
