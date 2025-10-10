#!/usr/bin/env python3
"""
ASCII checker for README files.
Ensures that README files contain only ASCII characters to prevent encoding issues.
"""

import sys
import os


def check_ascii(file_path):
    """Check if a file contains only ASCII characters."""
    if not os.path.exists(file_path):
        print(f"❌ File not found: {file_path}")
        return False

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    non_ascii_chars = []
    for line_num, line in enumerate(content.split('\n'), 1):
        for char_pos, char in enumerate(line, 1):
            if ord(char) > 127:
                non_ascii_chars.append({
                    'line': line_num,
                    'col': char_pos,
                    'char': char,
                    'code': ord(char)
                })

    if non_ascii_chars:
        print(f"❌ Non-ASCII characters found in {file_path}:")
        for item in non_ascii_chars[:10]:  # Show first 10
            print(f"   Line {item['line']}, Column {item['col']}: "
                  f"'{item['char']}' (U+{item['code']:04X})")
        if len(non_ascii_chars) > 10:
            print(f"   ... and {len(non_ascii_chars) - 10} more")
        return False

    print(f"✅ {file_path} contains only ASCII characters")
    return True


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 asciicheck.py <file_path> [<file_path2> ...]")
        sys.exit(1)

    all_passed = True
    for file_path in sys.argv[1:]:
        if not check_ascii(file_path):
            all_passed = False

    if not all_passed:
        sys.exit(1)


if __name__ == "__main__":
    main()
