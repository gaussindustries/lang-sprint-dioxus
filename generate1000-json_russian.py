#!/usr/bin/env python3
import json
from pathlib import Path

INPUT = Path("1000_russian.txt")
OUTPUT = Path("langs/russian/1000.json")

entries = []

with INPUT.open("r", encoding="utf-8") as f:
    for raw_line in f:
        line = raw_line.strip()
        if not line:
            continue

        # Split on tabs, then drop empty cells caused by double tabs
        cols = [c.strip() for c in line.split("\t") if c.strip()]

        if len(cols) < 3:
            print("Skipping weird line (needs at least rank, word, en):", repr(line))
            continue

        # cols should now look like:
        # ["1.", "и", "and, though", "conjunction"]
        rank_str = cols[0].rstrip(".")
        ru = cols[1]
        en = cols[2]
        pos = cols[3] if len(cols) >= 4 else None

        try:
            rank = int(rank_str)
        except ValueError:
            print("Skipping non-numeric rank:", repr(line))
            continue

        entry = {
            "rank": rank,
            "word": ru,   # Russian
            "en": en,     # English gloss
        }

        if pos:
            entry["pos"] = pos

        # example stays absent => Option::None in Rust
        entries.append(entry)

# Sort by rank just in case
entries.sort(key=lambda x: x["rank"])

OUTPUT.parent.mkdir(parents=True, exist_ok=True)
with OUTPUT.open("w", encoding="utf-8") as f:
    json.dump(entries, f, ensure_ascii=False, indent=2)

print(f"Wrote {len(entries)} entries to {OUTPUT}")
