#!/usr/bin/env python3
"""
Analyze tempo error ratios (pred/gt) for a validation_results_*.csv file.

This is a lightweight helper for quickly spotting metrical-level / harmonic confusions
(e.g., 2×, 1/2×, 3:2, 4:3) in validation outputs.
"""

import argparse
import csv
import os

from validation._metrics import TEMPO_RATIO_FACTORS, tempo_ratio_bucket


def main() -> None:
    parser = argparse.ArgumentParser(description="Analyze pred/gt ratio buckets in validation results")
    parser.add_argument(
        "--file",
        required=True,
        help="Path to validation_results_*.csv",
    )
    parser.add_argument(
        "--tol",
        type=float,
        default=0.08,
        help="Absolute tolerance around ratio factors (default: 0.08)",
    )
    args = parser.parse_args()

    path = args.file
    tol = float(args.tol)

    with open(path, "r", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))

    counts = {}
    n = 0
    for row in rows:
        bucket = tempo_ratio_bucket(row.get("bpm_pred"), row.get("bpm_gt"), tol)
        if bucket == "N/A":
            continue
        counts[bucket] = counts.get(bucket, 0) + 1
        n += 1

    print(f"File: {os.path.basename(path)}")
    print(f"n={n}")
    print("factors: " + ", ".join(f"{name}={factor:.3f}" for name, factor in TEMPO_RATIO_FACTORS))
    print(f"ratio buckets (±{tol}):")
    for k, v in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
        print(f"  {k}: {v}")


if __name__ == "__main__":
    main()

