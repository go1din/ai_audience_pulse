import logging
from ultralytics import YOLO
import cv2
import logging
import os
import pandas as pd
import duckdb
import re
from datetime import datetime, timezone

LINE_WIDTH = 1 
FONT_SIZE = 1  
CONFIDENCE_THRESHOLD = 0.2


def main():
    model_path =  '../../assets/yolov12n-face.pt'
    video_path = '../../assets/audienceVR.mp4'
    db_file_path = '../../runs/inference.duckdb'
    table_name = sanitize_path_to_table_name(video_path)
    model = load_model(model_path=model_path)
    results = inference_video(model, video_path, db_file_path, table_name)

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

def sanitize_path_to_table_name(video_path):
    """
    Creates a valid and descriptive SQL table name from a file path.
    Example: '../../assets/audienceVR.mp4' -> 'audienceVR_mp4'
    """
    base_name = os.path.basename(video_path)
    
    # Replace characters that are invalid in SQL table names (like dots and slashes)
    # We replace '.' with '_' to keep the file extension visible
    table_name = base_name.replace('.', '_')
    
    # Remove or replace any other non-alphanumeric characters 
    table_name = re.sub(r'[^\w_]', '', table_name)
    
    # Add a prefix to ensure the name doesn't start with a number 
    return f"inference_log_{table_name}".lower()

def save_to_duckdb(df, db_file_path, table_name):
    """
    Connects to a local DuckDB file and bulk inserts the data into the specified table.
    """
    logging.info(f"Connecting to DuckDB file: {db_file_path}")
    
    # Use a context manager to ensure the connection is closed
    with duckdb.connect(database=db_file_path) as con:
        try:
            # CREATE TABLE IF NOT EXISTS ensures the table structure is defined
            con.sql(f"CREATE TABLE IF NOT EXISTS {table_name} AS SELECT * FROM df WHERE 1=0")
            
            # Use INSERT INTO BY NAME for robustness
            con.sql(f"INSERT INTO {table_name} BY NAME SELECT * FROM df")

            logging.info(f"Bulk inserted {len(df)} rows into DuckDB table '{table_name}'.")
        except Exception as e:
            logging.error(f"Failed to bulk insert data into DuckDB: {e}")

def inference_video(model, video_path, db_file_path, table_name):
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
    processing_time = datetime.now(timezone.utc) 
    logging.info(f"Starting inference for '{video_path}' and logging to table: {table_name}")

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

    detection_data = []
    frame_number = 0

    while cap.isOpened():
        ret, frame = cap.read()
        if not ret: break
        frame_number += 1
        
        time_in_video_sec = frame_number / (fps if fps else 30)
        
        results = model(frame, conf=CONFIDENCE_THRESHOLD, verbose=False, stream=True)
        annotated_frame = frame.copy()
        
        for result in results:
            boxes = result.boxes
            for box in boxes:
                x1, y1, x2, y2 = box.xyxy[0].cpu().numpy().astype(int)
                confidence = float(box.conf[0].cpu().numpy())
                class_id = int(box.cls[0].cpu().numpy())
                class_name = model.names[class_id]
                
                # DATA COLLECTION: 
                detection_data.append({
                    'processing_timestamp': processing_time, 
                    'frame_id': frame_number,
                    'time_in_video_sec': time_in_video_sec,
                    'class_name': class_name,
                    'confidence': confidence,
                    'x_min': x1,
                    'y_min': y1,
                    'x_max': x2,
                    'y_max': y2,
                    'frame_width': frame_width,
                    'frame_height': frame_height
                })

            # Video Annotation
            annotated_frame = result.plot(
                img=annotated_frame, 
                line_width=LINE_WIDTH, 
                font_size=FONT_SIZE
            )
        
        out.write(annotated_frame)

       # Optional: display frame for real-time check (can slow down processing), comment out for speed if not needed or frontend available
        cv2.imshow(f'{model} Inference', annotated_frame)
        if cv2.waitKey(1) & 0xFF == ord('q'):
             break 

    cap.release()
    out.release()
    cv2.destroyAllWindows()
    if detection_data:
        df = pd.DataFrame(detection_data)
        save_to_duckdb(df, db_file_path, table_name)
    else:
        logging.warning(f"No detections were made for {table_name}, skipping DuckDB insertion.")

    logging.info(f"Processing complete! Output saved to: {output_path}")
    return [] 

if __name__ == "__main__":
    main()
