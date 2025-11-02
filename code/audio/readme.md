# AudioClassifier

A Python class for classifying audio clips into **clapping** and **laughter** categories using the YAMNet deep learning model.

---

## Features

- Classifies audio clips into **clapping** and **laughter** categories.
- Supports both raw WAV data (as NumPy arrays) and WAV files.
- Uses the pre-trained [YAMNet](https://tfhub.dev/google/yamnet/1) model for audio classification.

---

## Requirements

1. uv

Install the dependencies using:

```bash
uv run audio_classifer.py
```

---

## Output

The classifier returns an `AudioClassificationResult` object with the following scores:

- `clapping_score`: Confidence score for clapping (0 to 1).
- `laughter_score`: Confidence score for laughter (0 to 1).

Example output:

```json
{
    "clapping_score": 0.85,
    "laughter_score": 0.12
}
```

---

## Notes

- The input audio must be **16KHz** for accurate results.
- The classifier uses the following YAMNet class indexes:
  - Clapping: `[58, 61, 62]`
  - Laughter: `[13, 15, 16]`

---
