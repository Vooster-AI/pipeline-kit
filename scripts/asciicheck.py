#!/usr/bin/env python3
"""
ASCII checker for README files.
Ensures that README files contain only ASCII characters to prevent encoding issues.
Allows specific non-ASCII characters commonly used in documentation.
"""

import sys
import os


# Allowlist of non-ASCII characters permitted in README files
ALLOWED_CHARS = set([
    # Box drawing characters (for tree structures and diagrams)
    0x2500,  # ─ (BOX DRAWINGS LIGHT HORIZONTAL)
    0x2502,  # │ (BOX DRAWINGS LIGHT VERTICAL)
    0x250C,  # ┌ (BOX DRAWINGS LIGHT DOWN AND RIGHT)
    0x2510,  # ┐ (BOX DRAWINGS LIGHT DOWN AND LEFT)
    0x2514,  # └ (BOX DRAWINGS LIGHT UP AND RIGHT)
    0x2518,  # ┘ (BOX DRAWINGS LIGHT UP AND LEFT)
    0x251C,  # ├ (BOX DRAWINGS LIGHT VERTICAL AND RIGHT)
    0x2524,  # ┤ (BOX DRAWINGS LIGHT VERTICAL AND LEFT)
    0x252C,  # ┬ (BOX DRAWINGS LIGHT DOWN AND HORIZONTAL)
    0x2534,  # ┴ (BOX DRAWINGS LIGHT UP AND HORIZONTAL)
    0x253C,  # ┼ (BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL)

    # Double box drawing characters
    0x2550,  # ═ (BOX DRAWINGS DOUBLE HORIZONTAL)
    0x2551,  # ║ (BOX DRAWINGS DOUBLE VERTICAL)
    0x2554,  # ╔ (BOX DRAWINGS DOUBLE DOWN AND RIGHT)
    0x2557,  # ╗ (BOX DRAWINGS DOUBLE DOWN AND LEFT)
    0x255A,  # ╚ (BOX DRAWINGS DOUBLE UP AND RIGHT)
    0x255D,  # ╝ (BOX DRAWINGS DOUBLE UP AND LEFT)
    0x2560,  # ╠ (BOX DRAWINGS DOUBLE VERTICAL AND RIGHT)
    0x2563,  # ╣ (BOX DRAWINGS DOUBLE VERTICAL AND LEFT)
    0x2566,  # ╦ (BOX DRAWINGS DOUBLE DOWN AND HORIZONTAL)
    0x2569,  # ╩ (BOX DRAWINGS DOUBLE UP AND HORIZONTAL)
    0x256C,  # ╬ (BOX DRAWINGS DOUBLE VERTICAL AND HORIZONTAL)

    # Block elements
    0x2580,  # ▀ (UPPER HALF BLOCK)
    0x2584,  # ▄ (LOWER HALF BLOCK)
    0x2588,  # █ (FULL BLOCK)
    0x258C,  # ▌ (LEFT HALF BLOCK)
    0x2590,  # ▐ (RIGHT HALF BLOCK)
    0x2591,  # ░ (LIGHT SHADE)
    0x2592,  # ▒ (MEDIUM SHADE)
    0x2593,  # ▓ (DARK SHADE)

    # Arrows
    0x2190,  # ← (LEFTWARDS ARROW)
    0x2191,  # ↑ (UPWARDS ARROW)
    0x2192,  # → (RIGHTWARDS ARROW)
    0x2193,  # ↓ (DOWNWARDS ARROW)
    0x2194,  # ↔ (LEFT RIGHT ARROW)
    0x2195,  # ↕ (UP DOWN ARROW)
    0x21D0,  # ⇐ (LEFTWARDS DOUBLE ARROW)
    0x21D1,  # ⇑ (UPWARDS DOUBLE ARROW)
    0x21D2,  # ⇒ (RIGHTWARDS DOUBLE ARROW)
    0x21D3,  # ⇓ (DOWNWARDS DOUBLE ARROW)
    0x21D4,  # ⇔ (LEFT RIGHT DOUBLE ARROW)

    # Triangle arrows
    0x25C0,  # ◀ (BLACK LEFT-POINTING TRIANGLE)
    0x25B6,  # ▶ (BLACK RIGHT-POINTING TRIANGLE)
    0x25B2,  # ▲ (BLACK UP-POINTING TRIANGLE)
    0x25BC,  # ▼ (BLACK DOWN-POINTING TRIANGLE)
    0x25C4,  # ◄ (BLACK LEFT-POINTING POINTER)
    0x25BA,  # ► (BLACK RIGHT-POINTING POINTER)

    # Status symbols
    0x2713,  # ✓ (CHECK MARK)
    0x2714,  # ✔ (HEAVY CHECK MARK)
    0x2705,  # ✅ (WHITE HEAVY CHECK MARK)
    0x2717,  # ✗ (BALLOT X)
    0x2718,  # ✘ (HEAVY BALLOT X)
    0x274C,  # ❌ (CROSS MARK)
    0x26A0,  # ⚠ (WARNING SIGN)
    0x26A1,  # ⚡ (HIGH VOLTAGE SIGN)
    0x2139,  # ℹ (INFORMATION SOURCE)

    # Common emojis for README
    0x2764,  # ❤ (HEAVY BLACK HEART)
    0x1F4A1, # 💡 (ELECTRIC LIGHT BULB)
    0x1F680, # 🚀 (ROCKET)
    0x1F4E6, # 📦 (PACKAGE)
    0x1F4DD, # 📝 (MEMO)
    0x1F527, # 🔧 (WRENCH)
    0x1F3AF, # 🎯 (DIRECT HIT)
    0x1F525, # 🔥 (FIRE)
    0x2B50,  # ⭐ (WHITE MEDIUM STAR)
    0xFE0F,  # (VARIATION SELECTOR-16, for emoji rendering)

    # Bullets and symbols
    0x2022,  # • (BULLET)
    0x00B7,  # · (MIDDLE DOT)
    0x2605,  # ★ (BLACK STAR)
    0x2606,  # ☆ (WHITE STAR)
    0x25CB,  # ○ (WHITE CIRCLE)
    0x25CF,  # ● (BLACK CIRCLE)
    0x2122,  # ™ (TRADE MARK SIGN)
    0x00A9,  # © (COPYRIGHT SIGN)
    0x00AE,  # ® (REGISTERED SIGN)
])


def check_ascii(file_path):
    """Check if a file contains only ASCII characters (or allowed non-ASCII chars)."""
    if not os.path.exists(file_path):
        print(f"❌ File not found: {file_path}")
        return False

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    non_ascii_chars = []
    for line_num, line in enumerate(content.split('\n'), 1):
        for char_pos, char in enumerate(line, 1):
            char_code = ord(char)
            if char_code > 127 and char_code not in ALLOWED_CHARS:
                non_ascii_chars.append({
                    'line': line_num,
                    'col': char_pos,
                    'char': char,
                    'code': char_code
                })

    if non_ascii_chars:
        print(f"❌ Non-ASCII characters found in {file_path}:")
        for item in non_ascii_chars[:10]:  # Show first 10
            print(f"   Line {item['line']}, Column {item['col']}: "
                  f"'{item['char']}' (U+{item['code']:04X})")
        if len(non_ascii_chars) > 10:
            print(f"   ... and {len(non_ascii_chars) - 10} more")
        return False

    print(f"✅ {file_path} contains only ASCII characters (or allowed symbols)")
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
