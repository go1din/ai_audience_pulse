#include "esp_camera.h"
#include <WiFi.h>
#include <I2S.h>

#include "esp_http_client.h"

//
// WARNING!!! PSRAM IC required for UXGA resolution and high JPEG quality
//            Ensure ESP32 Wrover Module or other board with PSRAM is selected
//            Partial images will be transmitted if image exceeds buffer size
//
//            You must select partition scheme from the board menu that has at least 3MB APP space.
//            Face Recognition is DISABLED for ESP32 and ESP32-S2, because it takes up from 15
//            seconds to process single frame. Face Detection is ENABLED if PSRAM is enabled as well

// ===================
// Select camera model
// ===================
#define CAMERA_MODEL_XIAO_ESP32S3 // Has PSRAM
#include "camera_pins.h"

// ===========================
// Enter your WiFi credentials
// ===========================
const char *ssid = "memox.world";
const char *password = "worXperience!";

// Optional: define how many images to capture
#define MAX_IMAGES 1000

// make changes as needed
#define RECORD_TIME   2  // seconds, The maximum value is 240

// do not change for best
#define SAMPLE_RATE 16000U
#define SAMPLE_BITS 16
#define VOLUME_GAIN 2


void startCameraServer();
void setupLedFlash(int pin);
void record_audio();

#define MAX_HTTP_RECV_BUFFER 512
#define MAX_HTTP_OUTPUT_BUFFER 2048
static const char *TAG = "HTTP_CLIENT";

esp_err_t _http_event_handler(esp_http_client_event_t* evt)
{
  // Serial.printf("HTTP Event Performed %d \n", evt->event_id);

  switch(evt->event_id) {
      case HTTP_EVENT_ERROR:
          ESP_LOGI(TAG, "HTTP_EVENT_ERROR");
          break;
      case HTTP_EVENT_ON_CONNECTED:
          ESP_LOGI(TAG, "HTTP_EVENT_ON_CONNECTED");
          break;
      case HTTP_EVENT_HEADER_SENT:
          ESP_LOGI(TAG, "HTTP_EVENT_HEADER_SENT");
          break;
      case HTTP_EVENT_ON_HEADER:
          ESP_LOGI(TAG, "HTTP_EVENT_ON_HEADER, key=%s, value=%s", evt->header_key, evt->header_value);
          break;
      case HTTP_EVENT_ON_DATA:
          ESP_LOGI(TAG, "HTTP_EVENT_ON_DATA, len=%d", evt->data_len);
          //if (!esp_http_client_is_chunked_response(evt->client)) 
          //{
          // Serial.printf("HTTP Data Sent (data_len=%d)\n", evt->data_len);
          //}
          break;
      case HTTP_EVENT_ON_FINISH:
          ESP_LOGI(TAG, "HTTP_EVENT_ON_FINISH");
          break;
      case HTTP_EVENT_DISCONNECTED:
          ESP_LOGI(TAG, "HTTP_EVENT_DISCONNECTED");
          break;
      // case HTTP_EVENT_REDIRECT:
      //     ESP_LOGI(TAG, "HTTP_EVENT_REDIRECT");
      //     break;
  }
  return ESP_OK;
}

esp_http_client_config_t http_client_config = {
    .host = "http://192.168.20.166",
    .port = 5000,
    .path = "bmp",
    .event_handler = _http_event_handler,
    .is_async = false,
};


void setup() {
  Serial.begin(115200);
  Serial.setDebugOutput(true);
  Serial.println();

  camera_config_t config;
  config.ledc_channel = LEDC_CHANNEL_0;
  config.ledc_timer = LEDC_TIMER_0;
  config.pin_d0 = Y2_GPIO_NUM;
  config.pin_d1 = Y3_GPIO_NUM;
  config.pin_d2 = Y4_GPIO_NUM;
  config.pin_d3 = Y5_GPIO_NUM;
  config.pin_d4 = Y6_GPIO_NUM;
  config.pin_d5 = Y7_GPIO_NUM;
  config.pin_d6 = Y8_GPIO_NUM;
  config.pin_d7 = Y9_GPIO_NUM;
  config.pin_xclk = XCLK_GPIO_NUM;
  config.pin_pclk = PCLK_GPIO_NUM;
  config.pin_vsync = VSYNC_GPIO_NUM;
  config.pin_href = HREF_GPIO_NUM;
  config.pin_sccb_sda = SIOD_GPIO_NUM;
  config.pin_sccb_scl = SIOC_GPIO_NUM;
  config.pin_pwdn = PWDN_GPIO_NUM;
  config.pin_reset = RESET_GPIO_NUM;
  config.xclk_freq_hz = 20000000;
  config.frame_size = FRAMESIZE_FHD;
  config.pixel_format = PIXFORMAT_JPEG;  // for streaming
  //config.pixel_format = PIXFORMAT_RGB565; // for face detection/recognition
  config.grab_mode = CAMERA_GRAB_WHEN_EMPTY;
  config.fb_location = CAMERA_FB_IN_PSRAM;
  config.jpeg_quality = 12;
  config.fb_count = 1;

  // if PSRAM IC present, init with UXGA resolution and higher JPEG quality
  //                      for larger pre-allocated frame buffer.
  if (config.pixel_format == PIXFORMAT_JPEG) {
    if (psramFound()) {
      config.jpeg_quality = 10;
      config.fb_count = 2;
      config.grab_mode = CAMERA_GRAB_LATEST;
    } else {
      // Limit the frame size when PSRAM is not available
      config.frame_size = FRAMESIZE_SVGA;
      config.fb_location = CAMERA_FB_IN_DRAM;
    }
  } else {
    // Best option for face detection/recognition
    config.frame_size = FRAMESIZE_240X240;
#if CONFIG_IDF_TARGET_ESP32S3
    config.fb_count = 2;
#endif
  }

#if defined(CAMERA_MODEL_ESP_EYE)
  pinMode(13, INPUT_PULLUP);
  pinMode(14, INPUT_PULLUP);
#endif

  // camera init
  esp_err_t err = esp_camera_init(&config);
  if (err != ESP_OK) {
    Serial.printf("Camera init failed with error 0x%x", err);
    return;
  }

  sensor_t *s = esp_camera_sensor_get();
  // initial sensors are flipped vertically and colors are a bit saturated
//  if (s->id.PID == OV3660_PID) {
    s->set_vflip(s, 1);        // flip it back
    // s->set_brightness(s, 1);   // up the brightness just a bit
    // s->set_saturation(s, -2);  // lower the saturation

    // Try to improve inmage quality:
    s->set_quality(s, 10);      // High JPEG quality
    s->set_brightness(s, 1);    // Slightly brighter
    s->set_contrast(s, 1);      // More contrast
    s->set_saturation(s, 1);    // Richer colors
    s->set_whitebal(s, 1);      // Enable auto white balance

//  }
  // drop down frame size for higher initial frame rate
//  if (config.pixel_format == PIXFORMAT_JPEG) {
//    s->set_framesize(s, FRAMESIZE_QVGA);
//  }

#if defined(CAMERA_MODEL_M5STACK_WIDE) || defined(CAMERA_MODEL_M5STACK_ESP32CAM)
  s->set_vflip(s, 1);
  s->set_hmirror(s, 1);
#endif

#if defined(CAMERA_MODEL_ESP32S3_EYE)
  s->set_vflip(s, 1);
#endif

// Setup LED FLash if LED pin is defined in camera_pins.h
#if defined(LED_GPIO_NUM)
  setupLedFlash(LED_GPIO_NUM);
#endif

  WiFi.begin(ssid, password);
  WiFi.setSleep(false);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("");
  Serial.println("WiFi connected");

  // startCameraServer();

  Serial.print("Camera Ready! Use 'http://");
  Serial.print(WiFi.localIP());
  Serial.println("' to connect");

  // while (!Serial) ;
  // I2S.setAllPins(-1, 42, 41, -1, -1);
  // if (!I2S.begin(PDM_MONO_MODE, SAMPLE_RATE, SAMPLE_BITS)) {
  //   Serial.println("Failed to initialize I2S!");
  //   while (1) ;
  // }
}

void capture_images()
{
  esp_http_client_handle_t http_client = esp_http_client_init(&http_client_config);

  static int imageCount = 0;
  while(imageCount++ < MAX_IMAGES) 
  {
    uint64_t start_time = esp_timer_get_time();
    camera_fb_t *fb = esp_camera_fb_get();
    if (!fb) {
      Serial.println("Camera capture failed");
      exit;
    }

    // Image conversion logic would go here
    // uint8_t * buf = NULL;
    // size_t buf_len = 0;
    // bool converted = frame2jpg(fb, 0, &buf, &buf_len);
    // if(!converted){
    //     log_e("JPG Conversion failed");
    //     return;
    //   }

    // For simplicity, we just print the size
    Serial.printf("%lld - Captured image: %d bytes (format=%d , %d x %d , timestamp=%d \n", start_time, fb->len, fb->format, fb->width, fb->height, fb->timestamp);

    esp_http_client_set_url(http_client, "http://192.168.20.166:5000/img/1"); // TODO: change for every camera
    esp_http_client_set_method(http_client, HTTP_METHOD_POST);
    esp_http_client_set_header(http_client, "Content-Type", "image/x-windows-bmp");
    esp_http_client_set_post_field(http_client, (const char*)fb->buf, fb->len);

    esp_err_t err = esp_http_client_perform(http_client);

    if(err == ESP_OK) 
    {
      ESP_LOGI(TAG, "HTTP POST Status = %d, content_length = %lld",
            esp_http_client_get_status_code(http_client),
            esp_http_client_get_content_length(http_client));
    } 
    else 
    {
      ESP_LOGE(TAG, "HTTP POST request failed: %s", esp_err_to_name(err));
    }
    
    esp_camera_fb_return(fb);

    // // wait max 1 second between captures
    // uint64_t end_time = esp_timer_get_time();
    // uint64_t elapsed_time_msec = (uint64_t)((end_time - start_time) / 1000);
    // uint64_t remaining_time_msec = elapsed_time_msec < 1000 ? 1000 - elapsed_time_msec : 1;
    // log_i("BMP: %llums, %uB", elapsed_time, buf_len);
    // Serial.printf("Waiting for next image %d milliseconds\n", remaining_time_msec);
    // delay(remaining_time_msec);
  }

  esp_http_client_cleanup(http_client);
}


void record_audio()
{
  uint32_t sample_size = 0;
  uint32_t record_size = (SAMPLE_RATE * SAMPLE_BITS / 8) * RECORD_TIME;
  uint8_t *rec_buffer = NULL;
  Serial.printf("Ready to start recording ...\n");

  // PSRAM malloc for recording
  rec_buffer = (uint8_t *)ps_malloc(record_size);
  if (rec_buffer == NULL) {
    Serial.printf("malloc failed!\n");
    while(1) ;
  }
  Serial.printf("Buffer: %d bytes\n", ESP.getPsramSize() - ESP.getFreePsram());

  // Start recording
  esp_i2s::i2s_read(esp_i2s::I2S_NUM_0, rec_buffer, record_size, &sample_size, portMAX_DELAY);
  if (sample_size == 0) {
    Serial.printf("Record Failed!\n");
  } else {
    Serial.printf("Record %d bytes\n", sample_size);
  }

  // Increase volume
  for (uint32_t i = 0; i < sample_size; i += SAMPLE_BITS/8) {
    (*(uint16_t *)(rec_buffer+i)) <<= VOLUME_GAIN;
  }

  free(rec_buffer);
  Serial.printf("The recording is over.\n");
}


void loop() 
{
  // Do nothing. Everything is done in another task by the web server
  // delay(10000);

  capture_images();

  //record_audio();
}