import cv2
import mediapipe as mp #face detector
import math
import numpy as np
import time

# torch
import torch
from PIL import Image
from torchvision import transforms

### sub functions
def pth_processing(fp):
    class PreprocessInput(torch.nn.Module):
        def init(self):
            super(PreprocessInput, self).init()

        def forward(self, x):
            x = x.to(torch.float32)
            x = torch.flip(x, dims=(0,))
            x[0, :, :] -= 91.4953
            x[1, :, :] -= 103.8827
            x[2, :, :] -= 131.0912
            return x

    def get_img_torch(img):

        ttransform = transforms.Compose([
            transforms.PILToTensor(),
            PreprocessInput()
        ])
        img = img.resize((224, 224), Image.Resampling.NEAREST)
        img = ttransform(img)
        img = torch.unsqueeze(img, 0) #.to('cuda')
        return img
    return get_img_torch(fp)

def tf_processing(fp):
    def preprocess_input(x):
        x_temp = np.copy(x)
        x_temp = x_temp[..., ::-1]
        x_temp[..., 0] -= 91.4953
        x_temp[..., 1] -= 103.8827
        x_temp[..., 2] -= 131.0912
        return x_temp

    def get_img_tf(img):
        img = cv2.resize(img, (224,224), interpolation=cv2.INTER_NEAREST)
        img = tf.keras.utils.img_to_array(img)
        img = preprocess_input(img)
        img = np.array([img])
        return img

    return get_img_tf(fp)

def norm_coordinates(normalized_x, normalized_y, image_width, image_height):

    x_px = min(math.floor(normalized_x * image_width), image_width - 1)
    y_px = min(math.floor(normalized_y * image_height), image_height - 1)

    return x_px, y_px

def get_box(fl, w, h):
    idx_to_coors = {}
    for idx, landmark in enumerate(fl.landmark):
        landmark_px = norm_coordinates(landmark.x, landmark.y, w, h)

        if landmark_px:
            idx_to_coors[idx] = landmark_px

    x_min = np.min(np.asarray(list(idx_to_coors.values()))[:,0])
    y_min = np.min(np.asarray(list(idx_to_coors.values()))[:,1])
    endX = np.max(np.asarray(list(idx_to_coors.values()))[:,0])
    endY = np.max(np.asarray(list(idx_to_coors.values()))[:,1])

    (startX, startY) = (max(0, x_min), max(0, y_min))
    (endX, endY) = (min(w - 1, endX), min(h - 1, endY))

    return startX, startY, endX, endY

def display_EMO_PRED(img, box, label='', color=(128, 128, 128), txt_color=(255, 255, 255), line_width=2, ):
    lw = line_width or max(round(sum(img.shape) / 2 * 0.003), 2)
    text2_color = (255, 0, 255)
    p1, p2 = (int(box[0]), int(box[1])), (int(box[2]), int(box[3]))
    cv2.rectangle(img, p1, p2, text2_color, thickness=lw, lineType=cv2.LINE_AA)
    font = cv2.FONT_HERSHEY_SIMPLEX

    tf = max(lw - 1, 1)
    text_fond = (0, 0, 0)
    text_width_2, text_height_2 = cv2.getTextSize(label, font, lw / 3, tf)
    text_width_2 = text_width_2[0] + round(((p2[0] - p1[0]) * 10) / 360)
    center_face = p1[0] + round((p2[0] - p1[0]) / 2)

    cv2.putText(img, label,
                (center_face - round(text_width_2 / 2), p1[1] - round(((p2[0] - p1[0]) * 20) / 360)), font,
                lw / 3, text_fond, thickness=tf, lineType=cv2.LINE_AA)
    cv2.putText(img, label,
                (center_face - round(text_width_2 / 2), p1[1] - round(((p2[0] - p1[0]) * 20) / 360)), font,
                lw / 3, text2_color, thickness=tf, lineType=cv2.LINE_AA)
    return img

def display_FPS(img, text, margin=1.0, box_scale=1.0):
    img_h, img_w, _ = img.shape
    line_width = int(min(img_h, img_w) * 0.001)  # line width
    thickness = max(int(line_width / 3), 1)  # font thickness

    font_face = cv2.FONT_HERSHEY_SIMPLEX
    font_color = (0, 0, 0)
    font_scale = thickness / 1.5

    t_w, t_h = cv2.getTextSize(text, font_face, font_scale, None)[0]

    margin_n = int(t_h * margin)
    sub_img = img[0 + margin_n: 0 + margin_n + t_h + int(2 * t_h * box_scale),
              img_w - t_w - margin_n - int(2 * t_h * box_scale): img_w - margin_n]

    white_rect = np.ones(sub_img.shape, dtype=np.uint8) * 255

    img[0 + margin_n: 0 + margin_n + t_h + int(2 * t_h * box_scale),
    img_w - t_w - margin_n - int(2 * t_h * box_scale):img_w - margin_n] = cv2.addWeighted(sub_img, 0.5, white_rect, .5,
                                                                                          1.0)

    cv2.putText(img=img,
                text=text,
                org=(img_w - t_w - margin_n - int(2 * t_h * box_scale) // 2,
                     0 + margin_n + t_h + int(2 * t_h * box_scale) // 2),
                fontFace=font_face,
                fontScale=font_scale,
                color=font_color,
                thickness=thickness,
                lineType=cv2.LINE_AA,
                bottomLeftOrigin=False)

    return img

## webcam part
mp_face_mesh = mp.solutions.face_mesh

name = '0_66_49_wo_gl'

# torch
pth_model = torch.jit.load('face/torchscript_model_{0}.pth'.format(name)) #.to('cuda')
pth_model.eval()



# tf
# tf_model = tf.keras.models.load_model('models_EmoAffectnet/weights_{0}.h5'.format(name))

DICT_EMO = {0: 'Neutral', 1: 'Happiness', 2: 'Sadness', 3: 'Surprise', 4: 'Fear', 5: 'Disgust', 6: 'Anger'}

cap = cv2.VideoCapture(0)
w = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
h = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
fps = np.round(cap.get(cv2.CAP_PROP_FPS))

path_save_video = 'result.mp4'
vid_writer = cv2.VideoWriter(path_save_video, cv2.VideoWriter_fourcc(*'mp4v'), fps, (w, h))

with mp_face_mesh.FaceMesh(
max_num_faces=1,
refine_landmarks=False,
min_detection_confidence=0.5,
min_tracking_confidence=0.5) as face_mesh:

    while cap.isOpened():
        t1 = time.time()
        success, frame = cap.read()
        if frame is None: break

        frame_copy = frame.copy()
        frame_copy.flags.writeable = False
        frame_copy = cv2.cvtColor(frame_copy, cv2.COLOR_BGR2RGB)
        results = face_mesh.process(frame_copy)
        frame_copy.flags.writeable = True

        if results.multi_face_landmarks:
            for fl in results.multi_face_landmarks:
                startX, startY, endX, endY  = get_box(fl, w, h)
                cur_face = frame_copy[startY:endY, startX: endX]

                # torch
                cur_face = pth_processing(Image.fromarray(cur_face))
                output = torch.nn.functional.softmax(pth_model(cur_face), dim=1).cpu().detach().numpy()

                # tf
                # cur_face = tf_processing(cur_face)
                # output = tf_model(cur_face, training=False).numpy()

                cl = np.argmax(output)
                label = DICT_EMO[cl]
                frame = display_EMO_PRED(frame, (startX, startY, endX, endY), label, line_width=3)

        t2 = time.time()

        frame = display_FPS(frame, 'FPS: {0:.1f}'.format(1 / (t2 - t1)), box_scale=.5)

        vid_writer.write(frame)

        cv2.imshow('Webcam', frame)
        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

    vid_writer.release()
    cap.release()
    cv2.destroyAllWindows()
