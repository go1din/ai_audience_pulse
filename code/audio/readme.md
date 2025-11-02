# AudioClassifier

1. **AudioClassifier**: Classifies audio clips into **clapping** and **laughter** categories.
2. **AudioProcessor**: Processes real-time audio streams, computes RMS, and sends classification results to a specified endpoint.

---

## Features

- Classifies audio clips into **clapping** and **laughter** categories.
- Supports both raw WAV data (as NumPy arrays) and WAV files.
- Real-time audio processing with RMS calculation.
- Sends classification results to a specified HTTP endpoint.
- Uses the pre-trained [YAMNet](https://tfhub.dev/google/yamnet/1) model for audio classification.

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
- The `AudioProcessor` sends classification results to `http://192.168.20.166:5000/audio/score` by default.

---
