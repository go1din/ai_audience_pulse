from audio_classifier import AudioClassifier

classifier = AudioClassifier()
result = classifier.classification_score_wav_file(wav_file_name="../../assets/sample_audio/real_clapping.wav")
print(result)