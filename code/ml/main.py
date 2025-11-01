import logging
from ultralytics import YOLO
import cv2
import logging
import os

LINE_WIDTH = 1 
FONT_SIZE = 1  
CONFIDENCE_THRESHOLD = 0.2


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
    except Exception as e:
        print(f"ERROR: Unexpected error: {e}")


def inference_video(model, video_path):
    """
    Runs YOLO prediction frame-by-frame on the video to allow custom annotation size
    and saves the output to a new video file.
    """
    if model is None:
        logging.error("Inference skipped because the model failed to load.")
        return []

    if not os.path.exists(video_path):
        logging.error(f"Video file not found at path: {video_path}")
        return []

    logging.info(f"Starting frame-by-frame inference on video: {video_path}")
    output_path = f'../../runs/{video_path}'
    cap = cv2.VideoCapture(video_path)

    if not cap.isOpened():
        logging.error(f"Error: Could not open video file {video_path}")
        return []

    # Get video properties
    frame_width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    frame_height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    fps = int(cap.get(cv2.CAP_PROP_FPS))

    # Define the codec and create VideoWriter object
    fourcc = cv2.VideoWriter_fourcc(*'mp4v') 
    out = cv2.VideoWriter(output_path, fourcc, fps, (frame_width, frame_height))

    frame_number = 0
    while cap.isOpened():
        ret, frame = cap.read()
        
        if not ret:
            break 
        
        frame_number += 1
        
        
        results = model(frame, conf=CONFIDENCE_THRESHOLD, verbose=False, stream=True)
        
        annotated_frame = frame.copy() 
        
        # Process results and draw bounding boxes
        for result in results:
            annotated_frame = result.plot(
                img=annotated_frame, 
                line_width=LINE_WIDTH, 
                font_size=FONT_SIZE
            )
        
        out.write(annotated_frame)
        
        # Optional: display frame for real-time check (can slow down processing)
        cv2.imshow(f'{model} Inference', annotated_frame)
        if cv2.waitKey(1) & 0xFF == ord('q'):
             break

    cap.release()
    out.release()
    cv2.destroyAllWindows()
    
    logging.info(f"Processing complete! Output saved to: {output_path}")
    return [] 

if __name__ == "__main__":
    main()
