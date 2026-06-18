#!/usr/bin/env python3
"""
Parse Oracle user_source CSV output into individual SQL files.
Each package/function/procedure gets its own .sql file.
Also generates a mapping index markdown file.

Usage: python3 parse_plsql_source.py
"""

import csv
import os
import io
import datetime

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BASE_DIR = os.path.join(os.path.dirname(SCRIPT_DIR), "..", "docs", "plsql")
STEPS_DIR = os.path.expanduser(
    "~/.gemini/antigravity/brain/e102cab4-5347-4514-a25f-b9d724afb89e/.system_generated/steps"
)

# Input files from MCP query outputs
INPUT_FILES = {
    "PACKAGE": os.path.join(STEPS_DIR, "250", "output.txt"),
    "FUNCTION": os.path.join(STEPS_DIR, "251", "output.txt"),
    "PROCEDURE": os.path.join(STEPS_DIR, "252", "output.txt"),
    "PACKAGE BODY": os.path.join(STEPS_DIR, "261", "output.txt"),
}

# Output subdirectories
SUBDIRS = {
    "PACKAGE": "packages",
    "PACKAGE BODY": "package_bodies",
    "FUNCTION": "functions",
    "PROCEDURE": "procedures",
}


def parse_source_file(filepath, obj_type):
    """Parse CSV output into dict of {name: [(line, text), ...]}."""
    objects = {}
    if not os.path.isfile(filepath):
        print(f"  WARNING: File not found: {filepath}")
        return objects

    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    # Remove trailing summary lines like "16947 rows selected."
    lines = content.strip().split("\n")
    csv_lines = []
    for line in lines:
        if line.strip() and not line.strip().endswith("rows selected."):
            csv_lines.append(line)

    reader = csv.reader(io.StringIO("\n".join(csv_lines)))
    header = next(reader, None)
    if not header:
        return objects

    for row in reader:
        if len(row) < 4:
            continue
        name, rtype, line_no, text = row[0], row[1], row[2], row[3]
        if name not in objects:
            objects[name] = []
        objects[name].append((int(line_no), text))

    # Sort lines within each object
    for name in objects:
        objects[name].sort(key=lambda x: x[0])

    return objects


def write_sql_file(outdir, name, obj_type, lines):
    """Write a .sql file for a single PL/SQL object."""
    os.makedirs(outdir, exist_ok=True)
    filename = f"{name.lower()}.sql"
    filepath = os.path.join(outdir, filename)

    with open(filepath, "w", encoding="utf-8") as f:
        f.write(f"-- =============================================================\n")
        f.write(f"-- {obj_type}: {name}\n")
        f.write(f"-- Extracted: {datetime.datetime.now().isoformat()}\n")
        f.write(f"-- Source: EBANKBIDC_CAM@10.22.19.128:1521:DVNHT1\n")
        f.write(f"-- =============================================================\n\n")

        if obj_type == "PACKAGE":
            f.write(f"CREATE OR REPLACE ")
        elif obj_type == "PACKAGE BODY":
            f.write(f"CREATE OR REPLACE ")
        elif obj_type == "FUNCTION":
            f.write(f"CREATE OR REPLACE ")
        elif obj_type == "PROCEDURE":
            f.write(f"CREATE OR REPLACE ")

        for _, text in lines:
            f.write(text)

        f.write("\n/\n")

    return filepath, len(lines)


def main():
    print("=" * 60)
    print("PL/SQL Source Code Extractor")
    print(f"Output: {BASE_DIR}")
    print("=" * 60)

    all_objects = []  # (type, name, status, line_count, filepath)

    for obj_type, input_file in INPUT_FILES.items():
        subdir = SUBDIRS[obj_type]
        outdir = os.path.join(BASE_DIR, subdir)
        print(f"\nProcessing {obj_type} from {os.path.basename(input_file)}...")

        objects = parse_source_file(input_file, obj_type)
        print(f"  Found {len(objects)} {obj_type}(s)")

        for name, lines in sorted(objects.items()):
            filepath, line_count = write_sql_file(outdir, name, obj_type, lines)
            rel_path = os.path.relpath(filepath, BASE_DIR)
            all_objects.append((obj_type, name, line_count, rel_path))
            print(f"  ✓ {name} ({line_count} lines)")

    # Generate mapping index
    index_path = os.path.join(BASE_DIR, "INDEX.md")
    with open(index_path, "w", encoding="utf-8") as f:
        f.write("# BIDC PL/SQL Source Code Index\n\n")
        f.write(f"> **Database**: EBANKBIDC_CAM@10.22.19.128:1521:DVNHT1\n")
        f.write(f"> **Extracted**: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"> **Total Objects**: {len(all_objects)}\n\n")

        # Summary table
        type_counts = {}
        for obj_type, _, _, _ in all_objects:
            type_counts[obj_type] = type_counts.get(obj_type, 0) + 1
        f.write("## Summary\n\n")
        f.write("| Type | Count |\n")
        f.write("|------|-------|\n")
        for t, c in sorted(type_counts.items()):
            f.write(f"| {t} | {c} |\n")
        f.write(f"| **Total** | **{len(all_objects)}** |\n\n")

        # Detailed listing by type
        current_type = None
        for obj_type, name, line_count, rel_path in all_objects:
            if obj_type != current_type:
                current_type = obj_type
                f.write(f"---\n\n## {obj_type}S\n\n")
                f.write("| # | Name | Lines | File |\n")
                f.write("|---|------|-------|------|\n")
                idx = 0
            idx += 1
            f.write(f"| {idx} | `{name}` | {line_count} | [{os.path.basename(rel_path)}]({rel_path}) |\n")
        f.write("\n")

    print(f"\n{'=' * 60}")
    print(f"✓ Generated {len(all_objects)} SQL files")
    print(f"✓ Index: {index_path}")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
