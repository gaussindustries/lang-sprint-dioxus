import json
from pathlib import Path
#input can be gathered from https://1000mostcommonwords.com/
INPUT = Path("example.txt")
OUTPUT = Path("langs/")

entries = []

with INPUT.open("r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue

        # Split into 3 parts: rank, ka, en
        # Your file is "rank<TAB>Georgian<TAB>English...".
        parts = line.split("\t")
        if len(parts) < 3:
            # fallback if it's space-separated instead of tabs
            parts = line.split(maxsplit=2)

        if len(parts) < 3:
            print("Skipping weird line:", line)
            continue

        rank_str, ka, en = parts[0], parts[1], parts[2]

        try:
            rank = int(rank_str)
        except ValueError:
            print("Skipping non-numeric rank:", line)
            continue

        entries.append({
            "rank": rank,
            "ge": ka, #suffix for target language
            "en": en.strip()
        })

# Sort by rank just to be sure
entries.sort(key=lambda x: x["rank"])

OUTPUT.parent.mkdir(parents=True, exist_ok=True)
with OUTPUT.open("w", encoding="utf-8") as f:
    json.dump(entries, f, ensure_ascii=False, indent=2)

print(f"Wrote {len(entries)} entries to {OUTPUT}")
