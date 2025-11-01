import json
import numpy as np
from dataclasses import dataclass
import tensorflow as tf
import tensorflow_hub as hub
from scipy.io import wavfile

@dataclass
class AudioClassificationResult:
    """Container for classification scores."""
    clapping_score: float
    laughter_score: float

    def to_json_string(self):
        return json.dumps({
            "clapping_score": float(self.clapping_score),
            "laughter_score": float(self.laughter_score)
        })


class AudioClassifier:

    def __init__(self):
        self.model = hub.load('../../assets/yamnet-tensorflow2-yamnet-v1')

        self.clapping_class_indexes = [
            58,  # Clapping
            61,  # Cheering
            62,  # Applause
        ]

        self.laughter_class_indexes = [
            13,  # Laughter
            15,  # Giggle
            16,  # Snicker
        ]

    # wav data should be 16KHz
    def classification_score_wav_data(self, wav_data: np.ndarray) -> AudioClassificationResult:
        waveform = wav_data / tf.int16.max

        # Run the model, check the output.
        scores, embeddings, spectrogram = self.model(waveform)

        clapping_score = scores.numpy()[:, self.clapping_class_indexes].max()#mean(axis=0).max()
        laughter_score = scores.numpy()[:, self.laughter_class_indexes].max()#mean(axis=0).max()

        return AudioClassificationResult(
            clapping_score=clapping_score,
            laughter_score=laughter_score
        )

    def classification_score_wav_file(self, wav_file_name: str) -> AudioClassificationResult:
        sample_rate, wav_data = wavfile.read(wav_file_name, 'rb')
        print("Sample rate:", sample_rate)
        #sample_rate, wav_data = _ensure_sample_rate(wav_file_name)
        return self.classification_score_wav_data(wav_data)


