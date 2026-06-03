import unittest

from validation._metrics import (
    beat_f_measure,
    bpm_absolute_error,
    bpm_within_tolerance,
    downbeat_f_measure,
    evaluate_key_mirex,
    key_mirex_summary,
    match_events_with_tolerance,
    parse_float,
    parse_event_times,
    tempo_ratio,
    tempo_ratio_bucket,
    trim_event_times,
)


class ValidationMetricsTests(unittest.TestCase):
    def test_parse_float_rejects_empty_and_non_finite(self):
        self.assertIsNone(parse_float(""))
        self.assertIsNone(parse_float(None))
        self.assertIsNone(parse_float("nan"))
        self.assertIsNone(parse_float("inf"))
        self.assertEqual(parse_float("120.5"), 120.5)

    def test_bpm_metrics(self):
        self.assertEqual(bpm_absolute_error(121.5, 120.0), 1.5)
        self.assertTrue(bpm_within_tolerance(121.5, 120.0))
        self.assertFalse(bpm_within_tolerance(123.0, 120.0))
        self.assertAlmostEqual(tempo_ratio(180.0, 120.0), 1.5)
        self.assertEqual(tempo_ratio_bucket(180.0, 120.0), "3/2x")
        self.assertEqual(tempo_ratio_bucket(119.0, 120.0), "1x")
        self.assertEqual(tempo_ratio_bucket(0.0, 120.0), "N/A")

    def test_event_time_parsing_and_trimming(self):
        self.assertEqual(
            parse_event_times(["6.0", "nan", "", 1.0, None, "4.5"]),
            [1.0, 4.5, 6.0],
        )
        self.assertEqual(trim_event_times([1.0, 4.999, 5.0, 6.0]), [5.0, 6.0])

    def test_beat_f_measure_uses_one_to_one_tolerance_matching(self):
        metrics = beat_f_measure(
            reference_beats=[1.0, 2.0, 3.0],
            estimated_beats=[1.02, 2.2, 4.0],
        )

        self.assertEqual(metrics["matches"], 1)
        self.assertEqual(metrics["false_positives"], 2)
        self.assertEqual(metrics["false_negatives"], 2)
        self.assertAlmostEqual(metrics["precision"], 1.0 / 3.0)
        self.assertAlmostEqual(metrics["recall"], 1.0 / 3.0)
        self.assertAlmostEqual(metrics["f_measure"], 1.0 / 3.0)
        self.assertAlmostEqual(metrics["mean_abs_error_seconds"], 0.02)

    def test_beat_f_measure_can_trim_early_events(self):
        metrics = beat_f_measure(
            reference_beats=[1.0, 6.0],
            estimated_beats=[1.5, 6.05],
            trim_before_seconds=5.0,
        )

        self.assertEqual(metrics["matches"], 1)
        self.assertEqual(metrics["false_positives"], 0)
        self.assertEqual(metrics["false_negatives"], 0)
        self.assertAlmostEqual(metrics["f_measure"], 1.0)

    def test_event_matching_rejects_invalid_tolerance(self):
        with self.assertRaises(ValueError):
            match_events_with_tolerance([1.0], [1.0], tolerance_seconds=float("nan"))
        with self.assertRaises(ValueError):
            match_events_with_tolerance([1.0], [1.0], tolerance_seconds=-0.01)

    def test_downbeat_f_measure_uses_same_matching(self):
        metrics = downbeat_f_measure([5.0, 7.0], [5.06, 9.0])

        self.assertEqual(metrics["matches"], 1)
        self.assertAlmostEqual(metrics["precision"], 0.5)
        self.assertAlmostEqual(metrics["recall"], 0.5)
        self.assertAlmostEqual(metrics["f_measure"], 0.5)

    def test_mirex_key_categories(self):
        self.assertEqual(evaluate_key_mirex("C", "C")["category"], "correct")
        self.assertEqual(evaluate_key_mirex("G", "C")["category"], "fifth")
        self.assertEqual(evaluate_key_mirex("F", "C")["category"], "fifth")
        self.assertEqual(
            evaluate_key_mirex("F", "C", allow_descending_fifths=False)["category"],
            "other",
        )
        self.assertEqual(evaluate_key_mirex("Am", "C")["category"], "relative")
        self.assertEqual(evaluate_key_mirex("C", "Am")["category"], "relative")
        self.assertEqual(evaluate_key_mirex("Cm", "C")["category"], "parallel")
        self.assertEqual(evaluate_key_mirex("Db", "C#")["category"], "correct")
        self.assertEqual(evaluate_key_mirex("not-a-key", "C")["category"], "invalid")

    def test_mirex_summary(self):
        rows = [
            {"pred": "C", "ref": "C"},
            {"pred": "G", "ref": "C"},
            {"pred": "Am", "ref": "C"},
            {"pred": "Cm", "ref": "C"},
            {"pred": "D", "ref": "C"},
            {"pred": "not-a-key", "ref": "C"},
            {"pred": "C", "ref": ""},
        ]
        summary = key_mirex_summary(rows, "pred", "ref")
        self.assertEqual(summary["total"], 6)
        self.assertEqual(summary["counts"]["correct"], 1)
        self.assertEqual(summary["counts"]["fifth"], 1)
        self.assertEqual(summary["counts"]["relative"], 1)
        self.assertEqual(summary["counts"]["parallel"], 1)
        self.assertEqual(summary["counts"]["other"], 1)
        self.assertEqual(summary["counts"]["invalid"], 1)
        self.assertAlmostEqual(summary["weighted_percent"], 100.0 / 3.0)


if __name__ == "__main__":
    unittest.main()
