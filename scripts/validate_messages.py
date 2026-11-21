#!/usr/bin/env python3
"""
Telegram Markdown & Message Validator for Gordafarid
Validates user-facing message formatting, unescaped characters, and Markdown syntax.
Uses persistent SHA-256 caching so unmodified messages pass instantly.
"""

import os
import re
import sys
import json
import time
import hashlib
import argparse

# Regex for extracting raw string literals and format strings
RUST_RAW_STR_REGEX = re.compile(r'r#"(.*?)"#', re.DOTALL)
RUST_FORMAT_STR_REGEX = re.compile(r'format!\s*\(\s*"([^"]+)"', re.DOTALL)

SQL_KEYWORDS = re.compile(r'^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP)\b', re.IGNORECASE)

def is_sql(text):
    return bool(SQL_KEYWORDS.search(text.strip()))

def extract_rust_messages(src_dir):
    """Scans Rust files for UI messages and notices (skipping internal SQL queries)."""
    messages = []
    for root, _, files in os.walk(src_dir):
        # Skip database repository modules
        if os.path.basename(root) == 'db':
            continue

        for f in files:
            if f.endswith('.rs'):
                path = os.path.join(root, f)
                rel_path = os.path.relpath(path, src_dir)
                try:
                    with open(path, 'r', encoding='utf-8') as file:
                        content = file.read()
                except Exception:
                    continue

                # Find raw string blocks (help, start, notices)
                for idx, m in enumerate(RUST_RAW_STR_REGEX.finditer(content)):
                    raw = m.group(1).strip()
                    if is_sql(raw) or len(raw) < 15:
                        continue
                    key = f"{rel_path}:raw_{idx}"
                    messages.append((key, raw))

                # Find format! templates containing user-facing markup or emojis
                for idx, m in enumerate(RUST_FORMAT_STR_REGEX.finditer(content)):
                    raw = m.group(1).strip()
                    if is_sql(raw):
                        continue
                    if any(c in raw for c in ['*', '`', '_', '⚠️', '🚫', '🔇', '📌', '🛡', '🌸', '✅', '❌']):
                        key = f"{rel_path}:format_{idx}"
                        messages.append((key, raw))
    return messages

def compute_hash(text):
    return hashlib.sha256(text.encode('utf-8')).hexdigest()[:16]

def validate_markdown_syntax(text):
    """
    Validates Markdown entity balancing and formatting errors.
    Returns (True, None) or (False, error_description).
    """
    # 1. Check code blocks balancing (triple backticks)
    triple_backticks = text.count("```")
    if triple_backticks % 2 != 0:
        return False, "Unbalanced code blocks (``` count is odd)"

    # Remove code blocks for inline checks
    text_no_codeblocks = re.sub(r'```.*?```', '', text, flags=re.DOTALL)

    # 2. Check inline code backticks
    single_backticks = text_no_codeblocks.count("`")
    if single_backticks % 2 != 0:
        return False, "Unbalanced inline code backticks (` count is odd)"

    # Remove inline code for further symbol checks
    text_no_code = re.sub(r'`[^`]*`', '', text_no_codeblocks)

    # 3. Check bold * balancing
    asterisks = text_no_code.count("*")
    if asterisks % 2 != 0:
        return False, "Unbalanced bold asterisks (* count is odd)"

    # 4. Check spoiler || balancing
    spoilers = text_no_code.count("||")
    if spoilers % 2 != 0:
        return False, "Unbalanced spoiler bars (|| count is odd)"

    # 5. Check links [text](url)
    open_brackets = text_no_code.count("[")
    close_brackets = text_no_code.count("]")
    if open_brackets != close_brackets:
        return False, f"Unbalanced square brackets: {open_brackets} '[' vs {close_brackets} ']'"

    return True, None

def main():
    parser = argparse.ArgumentParser(description="Gordafarid Message & Markdown Syntax Validator")
    parser.add_argument("--src", default="src", help="Path to src directory")
    parser.add_argument("--cache-file", default=".messages-cache.json", help="Path to cache file")
    parser.add_argument("--force", action="store_true", help="Force validation ignoring cache")
    args = parser.parse_args()

    src_dir = os.path.abspath(args.src)
    if not os.path.isdir(src_dir):
        sys.exit(f"❌ Error: Source directory '{src_dir}' not found.")

    messages = extract_rust_messages(src_dir)
    if not messages:
        print("ℹ️ No message templates found in src/.")
        sys.exit(0)

    cached_hashes = {}
    if not args.force and os.path.isfile(args.cache_file):
        try:
            with open(args.cache_file, 'r', encoding='utf-8') as cf:
                data = json.load(cf)
                cached_hashes = data.get("hashes", {})
        except Exception:
            cached_hashes = {}

    to_test = []
    for key, text in messages:
        h = compute_hash(text)
        if key not in cached_hashes or cached_hashes[key] != h:
            to_test.append((key, text, h))

    total = len(messages)
    if not to_test:
        print(f"⚡ [PASS] All {total} user-facing messages validated via cache (0 diffs).")
        sys.exit(0)

    print(f"🔍 Validating {len(to_test)} new/modified message templates (out of {total} total)...")

    failed = []
    passed = 0
    new_cache = dict(cached_hashes)

    for key, text, h in to_test:
        ok, err = validate_markdown_syntax(text)
        if ok:
            passed += 1
            new_cache[key] = h
            print(f"  ✅ [OK] {key}")
        else:
            failed.append((key, err, text))
            print(f"  ❌ [FAIL] {key}: {err}")

    # Write back cache
    try:
        with open(args.cache_file, 'w', encoding='utf-8') as cf:
            json.dump({"updated_at": int(time.time()), "hashes": new_cache}, cf, indent=2)
    except Exception as e:
        print(f"⚠️ Warning: Could not write cache: {e}")

    if failed:
        print("\n" + "=" * 60)
        print(f"❌ VALIDATION FAILED: {len(failed)} message(s) contain syntax errors!")
        print("=" * 60)
        for key, err, text in failed:
            print(f"\nLocation: {key}")
            print(f"Error:    {err}")
            preview = text[:120].replace('\n', ' ')
            print(f"Content:  {preview}...")
        sys.exit(1)
    else:
        print(f"\n🎉 [SUCCESS] All {passed} messages passed Markdown & formatting validation!")

if __name__ == '__main__':
    main()
