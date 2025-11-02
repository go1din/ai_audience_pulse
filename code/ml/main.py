import logging
from ultralytics import YOLO
import cv2
import os
import pandas as pd
import duckdb
import re
from datetime import datetime, timezone
import time
import glob

LINE_WIDTH = 1
FONT_SIZE = 1
CONFIDENCE_THRESHOLD = 0.2

def main():
    detection_model_path = "../../assets/yolov12n-face.pt"
    image_path = "../../assets/sample_images/" 
    image_paths = get_ordered_image_paths(image_path) #this is a memory leak waiting to happen
    output_dir_path = "../../runs/annotated_images"
    db_file_path = "../../runs/inference.duckdb"
    model = load_detection_model(detection_model_path)
    inference_images(model, image_paths, db_file_path, "inference", output_dir_path)

def get_ordered_image_paths(directory_path):
    """
    Reads all .jpg files from the specified directory path, sorts them 
    alphabetically, and returns the full paths.

    Args:
        directory_path (str): The path to the directory containing the images.

    Returns:
        list: A sorted list of absolute paths to all JPG files.
    """
    # 1. Construct the pattern for all JPG files (case-insensitive glob)
    # glob.iglob is often used, but glob.glob is simpler here. 
    # Use os.path.join to ensure correct path separator usage.
    search_pattern = os.path.join(directory_path, "*.jpg") 
    
    # 2. Find all files matching the pattern
    # The glob module is effective for pattern matching file names.
    image_paths = glob.glob(search_pattern)
    
    # 3. Sort the paths to maintain sequential order (e.g., frame001.jpg, frame002.jpg)
    image_paths.sort()
    
    return image_paths

def load_detection_model(detection_model_path):
    try:
        model = YOLO(detection_model_path)
        logging.info("Model loaded successfully")
        return model
    except ValueError as e:
        print(f"ERROR: Failed to load model with ValueError: {e}")
        return None
    except Exception as e:
        print(f"ERROR: Unexpected error: {e}")

def save_to_duckdb(df, db_file_path, table_name):
    """Connects to a local DuckDB file and bulk inserts data.

    This function establishes a connection to a specified local DuckDB file
    and performs a bulk insertion of data from the input DataFrame into
    the designated table.

    Args:
        df (pandas.DataFrame or similar): The DataFrame containing the data
            to be inserted into the database.
        db_file_path (str): The file path for the local DuckDB database.
        table_name (str): The name of the table in the database to insert
            the data into.
    """
    logging.info(f"Connecting to DuckDB file: {db_file_path}")

    # Use a context manager to ensure the connection is closed
    with duckdb.connect(database=db_file_path) as con:
        try:
            # CREATE TABLE IF NOT EXISTS ensures the table structure is defined
            con.sql(
                f"CREATE TABLE IF NOT EXISTS {table_name} AS SELECT * FROM df WHERE 1=0"
            )

            # Use INSERT INTO BY NAME for robustness
            con.sql(f"INSERT INTO {table_name} BY NAME SELECT * FROM df")

            logging.info(
                f"Bulk inserted {len(df)} rows into DuckDB table '{table_name}'."
            )
        except Exception as e:
            logging.error(f"Failed to bulk insert data into DuckDB: {e}")

def inference_images(model, image_paths, db_file_path, table_name, output_dir_path):
    """Runs YOLO prediction frame-by-frame on images and saves annotated images.

    This function processes a sequence of images (frames) using a YOLO model
    to perform object detection, allowing for custom annotation size. The
    annotated images are then saved to a specified output directory.

    Args:
        model: The loaded YOLO model instance used for inference.
        image_paths (list of str): A list of file paths to the images (frames)
            to be processed.
        db_file_path (str): The file path for the SQLite database where
            prediction results might be stored (though saving logic is not
            shown, this argument suggests its purpose).
        table_name (str): The name of the table within the database to be used.
        output_dir_path (str): The path to the directory where the annotated
            images will be saved.
    """
    if model is None:
        logging.error("Inference skipped because the model failed to load.")
        return []

    if not image_paths:
        logging.error("Input image_paths list is empty.")
        return []

    input_directory = os.path.dirname(image_paths[0])
    
    # Use the directory name (e.g., 'sample_images') as the base name for the whole batch/job.
    base_output_name = os.path.basename(input_directory)
    # --- Setup Output Directory ---
    os.makedirs(output_dir_path, exist_ok=True)
    logging.info(f"Saving annotated images to directory: {output_dir_path}")

    logging.info(f"Starting frame-by-frame inference on {len(image_paths)} images.")
    processing_time = datetime.now(timezone.utc)
    logging.info(
        f"Starting inference for image sequence and logging to table: {table_name}"
    )

    # Read the first image to get dimensions for logging/initial setup
    first_frame = cv2.imread(image_paths[0])
    if first_frame is None:
        logging.error(f"Error: Could not open the first image file {image_paths[0]}")
        return []

    frame_height, frame_width, _ = first_frame.shape
    
    # In a video, we skip frames. For images, we can choose to process all or every Nth.
    # For a sequence of images representing a video, we keep the frame skip logic.
    # Since we don't have FPS, we can't calculate a 1-second interval, 
    # so FRAME_SKIP will default to 1 (process all) or a hardcoded value.
    FRAME_SKIP = 1 # Process every image by default
    
    # If the image sequence represents a video, you might still want to skip some.
    # To maintain approximate 1-second logic (assuming 30 FPS source):
    # FRAME_SKIP = 30 
    # logging.info(f"Processing will run every {FRAME_SKIP} images (assuming 30 FPS source).")


    detection_data = []
    last_known_detections = []
    yolo_results = []
    last_plot_result = None
    con = duckdb.connect(database=db_file_path)
    
    try:
        # DuckDB table setup (remains the same)
        column_names = [
            'processing_timestamp', 'image_path', 'frame_id', 'time_in_video_sec',
            'class_name', 'confidence', 'x_min', 'y_min', 'x_max', 'y_max',
            'frame_width', 'frame_height'
        ]

        schema_placeholder_data = {
            'processing_timestamp': datetime.now(timezone.utc),
            'image_path': '',
            'frame_id': 0,
            'time_in_video_sec': 0.0,
            'class_name': '',
            'confidence': 0.0,
            'x_min': 0, 'y_min': 0, 'x_max': 0, 'y_max': 0,
            'frame_width': 0, 'frame_height': 0
        }

        temp_df_schema = pd.DataFrame([schema_placeholder_data], columns=column_names)
        con.register("temp_df_schema", temp_df_schema)
        con.sql(
            f"CREATE TABLE IF NOT EXISTS {table_name} AS SELECT * FROM temp_df_schema WHERE 1=0"
        )
        logging.info(f"DuckDB table '{table_name}' ensured to exist.")
    except Exception as e:
        logging.error(f"Failed to setup DuckDB table: {e}")
        con.close()
        return []

    # Start timer for commit logic
    last_commit_time = time.time()
    COMMIT_INTERVAL_SEC = 3
    
    # --- Main Image Processing Loop ---
    for frame_number, image_path in enumerate(image_paths, start=1):
        frame = cv2.imread(image_path)
        if frame is None:
            logging.warning(f"Skipping unreadable image: {image_path}")
            continue

        # time_in_video_sec calculation is now an approximation, assuming a fixed FPS (e.g., 30)
        # We can use the frame_number directly as the primary time index.
        time_in_video_sec = frame_number / 30.0 # Approximation using 30 FPS

        # Use image_path as a stand-in for image_path 
        image_name = os.path.basename(image_path)
        
        # --- Inference Logic ---
        if frame_number == 1 or (frame_number % FRAME_SKIP == 0):
            logging.debug(f"Running inference on image {frame_number} ({image_name})...")
            # YOLO model call remains the same, but 'frame' is the image data
            yolo_results = model(
                frame, conf=CONFIDENCE_THRESHOLD, verbose=False, stream=True
            )
            last_known_detections = []

            # Process results, extract data, and store for subsequent images (or just this one)
            for result in yolo_results:
                if last_plot_result is None:
                    last_plot_result = result

                last_known_detections.append(result.boxes)

                # Extract data for DuckDB logging (only log the images where inference runs)
                boxes = result.boxes
                for box in boxes:
                    x1, y1, x2, y2 = box.xyxy[0].cpu().numpy().astype(int)
                    confidence = float(box.conf[0].cpu().numpy())
                    class_id = int(box.cls[0].cpu().numpy())
                    class_name = model.names[class_id]

                    # Store data
                    detection_data.append(
                        {
                            "processing_timestamp": processing_time,
                            "image_path": image_path, # Log the individual image path
                            "frame_id": frame_number,
                            "time_in_video_sec": time_in_video_sec,
                            "class_name": class_name,
                            "confidence": confidence,
                            "x_min": x1,
                            "y_min": y1,
                            "x_max": x2,
                            "y_max": y2,
                            "frame_width": frame_width,
                            "frame_height": frame_height,
                        }
                    )

        # --- ANNOTATION: Use the LAST KNOWN detections for ALL images ---
        annotated_frame = frame.copy()

        # Manually draw the last known bounding boxes on the current frame
        if last_known_detections and last_plot_result is not None:
            # Need a check because last_known_detections is a list of Boxes objects
            if last_known_detections[0].data.numel() > 0:
                last_plot_result.boxes = last_known_detections[0]

            # Use the plot method to draw on the image
            annotated_frame = last_plot_result.plot(
                img=annotated_frame, line_width=LINE_WIDTH, font_size=FONT_SIZE
            )

        # --- Output Image Writing ---
        # The output file name is derived from the input image name
        output_path = os.path.join(output_dir_path, f"annotated_{image_name}")
        cv2.imwrite(output_path, annotated_frame)

        # --- DuckDB Commit Logic ---
        current_time = time.time()
        if current_time - last_commit_time >= COMMIT_INTERVAL_SEC and detection_data:
            df_commit = pd.DataFrame(detection_data)
            try:
                con.register("df_commit", df_commit)
                con.sql(f"INSERT INTO {table_name} BY NAME SELECT * FROM df_commit")
                con.commit()
                logging.info(
                    f"Committed {len(df_commit)} rows to DuckDB (3-second interval)."
                )
                detection_data = []
                last_commit_time = current_time
            except Exception as e:
                logging.error(f"Failed to commit data to DuckDB at interval: {e}")

        # Optional: display frame for real-time check (can slow down processing)
        cv2.imshow(f"{model} Inference", annotated_frame)
        if cv2.waitKey(1) & 0xFF == ord("q"):
            break

    # --- Cleanup ---
    cv2.destroyAllWindows()
    
    # Final DuckDB commit (same as before)
    if detection_data:
        df = pd.DataFrame(detection_data)
        try:
            con.register("df", df)
            con.sql(f"INSERT INTO {table_name} BY NAME SELECT * FROM df")
            con.commit()
            logging.info(
                f"Final commit: Bulk inserted {len(df)} remaining rows into DuckDB table '{table_name}'."
            )
        except Exception as e:
            logging.error(f"Failed to bulk insert final data into DuckDB: {e}")
    else:
        logging.warning(
            f"No remaining detections were made for {table_name} during the last interval."
        )

    con.close()
    logging.info(f"Processing complete! Annotated images saved to: {output_dir_path}")
    return []

if __name__ == "__main__":
    main()
