import sounddevice as sd
import numpy as np
import queue
import threading
import time
import tensorflow as tf
from audio_classifier import AudioClassifier

class AudioProcessor:
    def __init__(self):
        self.classifier = AudioClassifier()

    def process(self, audio: np.ndarray, sr: int):
        """
        audio: float32 array shape (samples, channels) for 1 second
        sr: sample rate (e.g., 16000 or 48000)
        """
        # Example: compute RMS and print it (replace with your logic)
        rms = np.sqrt(np.mean(audio**2))
        print(f"[proc] chunk rms={rms:.4f}, shape={audio.shape}, sr={sr}")
        #sd.write("output.wav", audio, sr)
        print(self.classifier.classification_score_wav_data(audio * tf.int16.max))


def stream_audio_to_processor(processor: AudioProcessor,
                              sr: int = 16000,
                              channels: int = 1,
                              seconds_per_chunk: float = 1.0,
                              device: int | None = None):
    """
    Records from the default (or specified) microphone in 1-second chunks
    and sends each chunk to processor.process(audio, sr).
    """
    q = queue.Queue()

    # Worker thread: take chunks from the queue and process them
    def worker():
        while True:
            chunk = q.get()
            if chunk is None:
                break
            try:
                mono = chunk[:, 0].copy()

                processor.process(mono, sr)
            finally:
                q.task_done()

    worker_thread = threading.Thread(target=worker, daemon=True)
    worker_thread.start()

    frames_per_chunk = int(sr * seconds_per_chunk)

    # The callback fires for every block (1 second here)
    def callback(indata, frames, time_info, status):
        if status:
            print("[stream]", status)
        # Make a copy because indata is reused by sounddevice
        q.put(indata.copy())

    # Open a continuous stream to avoid gaps between chunks
    with sd.InputStream(samplerate=sr,
                        channels=channels,
                        dtype="float32",
                        blocksize=frames_per_chunk,
                        callback=callback,
                        device=device):
        print("🎤 Recording… Press Ctrl+C to stop.")
        try:
            while True:
                time.sleep(0.1)
        except KeyboardInterrupt:
            print("\nStopping…")

    # Clean shutdown
    q.put(None)
    worker_thread.join(timeout=2)

if __name__ == "__main__":
    # OPTIONAL: pick a device (list with sd.query_devices())
    # import pprint; pprint.pp(sd.query_devices())
    # sd.default.device = (None, 1)  # (output, input) indices; set input index as needed

    processor = AudioProcessor()
    stream_audio_to_processor(processor, sr=16000, channels=1)
