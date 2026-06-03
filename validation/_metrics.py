from __future__ import annotations

import math
from typing import Iterable

from validation._keys import key_name_to_echonest_key_mode, normalize_key


TEMPO_RATIO_FACTORS: tuple[tuple[str, float], ...] = (
    ("1x", 1.0),
    ("2x", 2.0),
    ("1/2x", 0.5),
    ("3/2x", 1.5),
    ("2/3x", 2 / 3),
    ("4/3x", 4 / 3),
    ("3/4x", 3 / 4),
)

MIREX_KEY_SCORES = {
    "correct": 1.0,
    "fifth": 0.5,
    "relative": 0.3,
    "parallel": 0.2,
    "other": 0.0,
    "invalid": 0.0,
}

DEFAULT_BEAT_TOLERANCE_SECONDS = 0.07


def parse_float(value) -> float | None:
    """Return a finite float, or None if the value is empty/non-finite."""
    if value is None or value == "":
        return None
    try:
        parsed = float(value)
    except (TypeError, ValueError):
        return None
    if not math.isfinite(parsed):
        return None
    return parsed


def bpm_absolute_error(predicted, reference) -> float | None:
    pred = parse_float(predicted)
    ref = parse_float(reference)
    if pred is None or ref is None:
        return None
    return abs(pred - ref)


def bpm_within_tolerance(predicted, reference, tolerance: float = 2.0) -> bool:
    err = bpm_absolute_error(predicted, reference)
    return err is not None and err <= tolerance


def tempo_ratio(predicted, reference) -> float | None:
    pred = parse_float(predicted)
    ref = parse_float(reference)
    if pred is None or ref is None or pred <= 0.0 or ref <= 0.0:
        return None
    return pred / ref


def tempo_ratio_bucket(predicted, reference, tolerance: float = 0.08) -> str:
    ratio = tempo_ratio(predicted, reference)
    if ratio is None:
        return "N/A"
    for name, factor in TEMPO_RATIO_FACTORS:
        if abs(ratio - factor) <= tolerance:
            return name
    return "other"


def parse_event_times(values: Iterable[object] | None) -> list[float]:
    """Return finite event times in ascending order."""
    if values is None:
        return []
    parsed: list[float] = []
    for value in values:
        time = parse_float(value)
        if time is not None:
            parsed.append(time)
    return sorted(parsed)


def trim_event_times(
    values: Iterable[object] | None,
    min_time_seconds: float = 5.0,
) -> list[float]:
    """Return finite event times at or after `min_time_seconds`."""
    return [
        time
        for time in parse_event_times(values)
        if time >= min_time_seconds
    ]


def match_events_with_tolerance(
    reference_times: Iterable[object] | None,
    estimated_times: Iterable[object] | None,
    tolerance_seconds: float = DEFAULT_BEAT_TOLERANCE_SECONDS,
) -> dict[str, object]:
    """
    Match two event-time sequences with one-to-one tolerance-window matching.

    The default tolerance follows `mir_eval.beat.f_measure`'s 0.07 second
    window. This helper does not trim early events; call `trim_event_times` or
    use `beat_f_measure(..., trim_before_seconds=5.0)` when a benchmark protocol
    requires the common mir_eval beat preprocessing step.
    """
    if tolerance_seconds < 0.0 or not math.isfinite(tolerance_seconds):
        raise ValueError("tolerance_seconds must be a finite non-negative value")

    reference = parse_event_times(reference_times)
    estimated = parse_event_times(estimated_times)

    ref_idx = 0
    est_idx = 0
    matches = 0
    errors: list[float] = []

    while ref_idx < len(reference) and est_idx < len(estimated):
        diff = estimated[est_idx] - reference[ref_idx]
        if abs(diff) <= tolerance_seconds:
            matches += 1
            errors.append(diff)
            ref_idx += 1
            est_idx += 1
        elif estimated[est_idx] < reference[ref_idx] - tolerance_seconds:
            est_idx += 1
        else:
            ref_idx += 1

    false_positives = len(estimated) - matches
    false_negatives = len(reference) - matches
    precision = matches / len(estimated) if estimated else 0.0
    recall = matches / len(reference) if reference else 0.0
    f_measure = (
        2.0 * precision * recall / (precision + recall)
        if precision + recall > 0.0
        else 0.0
    )
    mean_abs_error = (
        sum(abs(error) for error in errors) / len(errors)
        if errors
        else None
    )

    return {
        "matches": matches,
        "false_positives": false_positives,
        "false_negatives": false_negatives,
        "precision": precision,
        "recall": recall,
        "f_measure": f_measure,
        "mean_abs_error_seconds": mean_abs_error,
    }


def beat_f_measure(
    reference_beats: Iterable[object] | None,
    estimated_beats: Iterable[object] | None,
    tolerance_seconds: float = DEFAULT_BEAT_TOLERANCE_SECONDS,
    trim_before_seconds: float | None = None,
) -> dict[str, object]:
    """Return beat precision/recall/F-measure with mir_eval's default window."""
    reference = (
        trim_event_times(reference_beats, trim_before_seconds)
        if trim_before_seconds is not None
        else parse_event_times(reference_beats)
    )
    estimated = (
        trim_event_times(estimated_beats, trim_before_seconds)
        if trim_before_seconds is not None
        else parse_event_times(estimated_beats)
    )
    return match_events_with_tolerance(reference, estimated, tolerance_seconds)


def downbeat_f_measure(
    reference_downbeats: Iterable[object] | None,
    estimated_downbeats: Iterable[object] | None,
    tolerance_seconds: float = DEFAULT_BEAT_TOLERANCE_SECONDS,
    trim_before_seconds: float | None = None,
) -> dict[str, object]:
    """Return downbeat precision/recall/F-measure using the beat tolerance."""
    return beat_f_measure(
        reference_downbeats,
        estimated_downbeats,
        tolerance_seconds,
        trim_before_seconds,
    )


def _key_tuple(key_name: str) -> tuple[int, int] | None:
    normalized = normalize_key(key_name)
    if not normalized:
        return None
    return key_name_to_echonest_key_mode(normalized)


def _relative_tonic(tonic: int, mode: int) -> int:
    # C major -> A minor; A minor -> C major.
    return (tonic - 3) % 12 if mode == 1 else (tonic + 3) % 12


def evaluate_key_mirex(
    predicted: str,
    reference: str,
    *,
    allow_descending_fifths: bool = True,
) -> dict[str, object]:
    """
    Evaluate a predicted key against a reference key using MIREX-style categories.

    Returns a dict with `category`, `score`, `predicted`, and `reference`.
    Scores follow the common MIREX key task weights:
    correct=1.0, fifth=0.5, relative=0.3, parallel=0.2, other=0.0.
    """
    pred_norm = normalize_key(predicted)
    ref_norm = normalize_key(reference)
    pred = _key_tuple(pred_norm)
    ref = _key_tuple(ref_norm)
    if pred is None or ref is None:
        return {
            "category": "invalid",
            "score": MIREX_KEY_SCORES["invalid"],
            "predicted": pred_norm,
            "reference": ref_norm,
        }

    pred_tonic, pred_mode = pred
    ref_tonic, ref_mode = ref

    if pred_tonic == ref_tonic and pred_mode == ref_mode:
        category = "correct"
    elif pred_mode == ref_mode and pred_tonic == (ref_tonic + 7) % 12:
        category = "fifth"
    elif (
        allow_descending_fifths
        and pred_mode == ref_mode
        and pred_tonic == (ref_tonic - 7) % 12
    ):
        category = "fifth"
    elif pred_mode != ref_mode and pred_tonic == _relative_tonic(ref_tonic, ref_mode):
        category = "relative"
    elif pred_tonic == ref_tonic and pred_mode != ref_mode:
        category = "parallel"
    else:
        category = "other"

    return {
        "category": category,
        "score": MIREX_KEY_SCORES[category],
        "predicted": pred_norm,
        "reference": ref_norm,
    }


def key_mirex_summary(rows: Iterable[dict], pred_field: str, ref_field: str) -> dict[str, object]:
    counts = {name: 0 for name in MIREX_KEY_SCORES}
    total = 0
    score_sum = 0.0
    for row in rows:
        if not normalize_key(row.get(ref_field, "")):
            continue
        evaluation = evaluate_key_mirex(row.get(pred_field, ""), row.get(ref_field, ""))
        category = str(evaluation["category"])
        counts[category] += 1
        total += 1
        score_sum += float(evaluation["score"])
    weighted = (score_sum / total * 100.0) if total else 0.0
    return {"total": total, "counts": counts, "weighted_percent": weighted}
