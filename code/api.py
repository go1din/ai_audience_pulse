from flask import Flask, request, jsonify, send_file
from flask_cors import CORS
from PIL import Image
import io
from matplotlib import pyplot as plt
import matplotlib
import struct
import numpy as np

# should fix threading issues
matplotlib.use('Agg')

app = Flask(__name__)
CORS(app)

image_handler = dict()

@app.route('/img/<id>', methods=['GET'])
def serve_image(id=0):
    global image_handler
    if(id in image_handler):
        return send_file(io.BytesIO(image_handler[id]), mimetype='image/jpeg')
    else:
        return 'Resource not available', 400



@app.route('/img/<id>', methods=['POST'])
def img(id=None):
    try:
        global image_handler
        # Read raw binary data from request body
        image_data = request.get_data()
        
        if not image_data:
            return jsonify({"error": "No image data provided"}), 400
        
        # Open image from binary data
        image_handler[id] = image_data

        return jsonify({
            "message": "Image received successfully"
        }), 200
    
    except Exception as e:
        return jsonify({"error": f"Failed to process image: {str(e)}"}), 400

json_file = None

@app.route('/audio/score', methods=['POST'])
def receive_json():
    if request.is_json:
        global json_file
        json_file = request.get_json()
        print(json_file)  # Process the received JSON data
        return 'JSON received!', 200
    else:
        return 'Request was not JSON', 400

@app.route('/audio/score', methods=['GET'])
def send_json():
    return jsonify(json_file)

if __name__ == '__main__':
    app.run(debug=True, port=5000, host='0.0.0.0')
