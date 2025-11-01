from flask import Flask, request, jsonify
from PIL import Image
import io
from matplotlib import pyplot as plt

app = Flask(__name__)

@app.route('/bmp', methods=['POST'])
def bmp():
    try:
        # Read raw binary data from request body
        image_data = request.get_data()
        
        if not image_data:
            return jsonify({"error": "No image data provided"}), 400
        
        # Open image from binary data
        image = Image.open(io.BytesIO(image_data))
        plt.figure()
        plt.imshow(image)
        plt.show()
        
        return jsonify({
            "message": "Image received successfully"
        }), 200
    
    except Exception as e:
        return jsonify({"error": f"Failed to process image: {str(e)}"}), 400

if __name__ == '__main__':
    app.run(debug=True, port=5000)
