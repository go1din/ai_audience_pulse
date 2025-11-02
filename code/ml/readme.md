# Machine Learning

The machine learning part consists of 2 pieces that we havent been able to connect timewise in the hackathon:
1. A face detection model
2. An emotion detection classifier



## Features

- **Face Detection**: Uses YOLOv12 for real-time face detection.
- **Annotation**: Draws bounding boxes and labels on detected faces.
- **DuckDB Integration**: Logs detection metadata (timestamps, coordinates, confidence) for analysis.
- **Configurable**: Adjust confidence thresholds, frame skipping, and output paths.

## Pipeline model:

The pipeline implemented in the main.py consists of the following:
- Loading a YOLOv12 face detection model.
- Processing a sequence of images (e.g., frames from a video or standalone images).
- Annotating detected faces with bounding boxes.
- Logging detection results (coordinates, confidence, class) to a **DuckDB** database.
- Saving annotated images to a specified output directory.

## Setup

### Prerequisites

Have uv installed.

### How to run

To run this you need to be in ML folder and execute:

```bash
uv run main.py
```
### Input & Output

1. Place your images in '../../assets/sample_images/'.
2. Ensure your model (yolov12n-face.pt) is in '../../assets/'.
3. Annotated images are saved to ../../runs/annotated_images/.
4. Data: Detection logs are saved to ../../runs/inference.duckdb in the table named inference.


## TODOs
- [ ] refactor face.py and make it modular to be reused later in combo with the main.py
- [ ] crop faces so we can classify emotion of each face
- [ ] Integrate the face.py into main.py
- [ ] integrate ml with frontend and remote cameras somehow

## Further ideas

- [ ] out of the saved inference metrics in duckdb you could generate a report of how good your
- [ ] An earlier version(can look through the commit history of main.py) of this pipeline read in mp4 files e.g. if you recorded an audience while you were presenting you can run it through to get an analysis if people were happy, suprised, ...) etc.
- [ ] you could also overlay the inference results with the metrics to generate/derive some sort of KPI.
