import logging
from ultralytics import YOLO
import cv2
import logging

def main():
    model_path =  '../../assets/yolov12n-face.pt'
    video_path = '../../assets/audienceVR.mp4'
    model = load_model(model_path=model_path)
    results = inference_video(model, video_path)


def load_model(model_path):
    try:
        model = YOLO(model_path) 
        logging.info("Model loaded successfully")
        return model
    except ValueError as e:
        print(f"ERROR: Failed to load model with ValueError: {e}")
        return None

def inference_video(model, video_path):
    results = model.predict(source=video_path, show=True, conf=0.5, save=True)


if __name__ == "__main__":
    main()
