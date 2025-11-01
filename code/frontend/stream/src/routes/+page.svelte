<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import confetti from 'canvas-confetti';
  import { initChart, updateChart } from '$lib/chart';
  import { connectWebSocket } from '$lib/websocket';
  import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';
  import Alert from '$lib/components/Alert.svelte';
  import Stats from '$lib/components/Stats.svelte';
  import FloatingReactions from '$lib/components/FloatingReactions.svelte';

  let stream: MediaStream;
  let videoRef: HTMLVideoElement;
  let chartCanvas: HTMLCanvasElement;
  let emojiCounts = { thumbs: 0, applause: 0, smile: 0 };
  let isLoading = true;
  let error: string | null = null;
  let isRecording = false;
  let recordingStartTime: number | null = null;
  let elapsedTime = '00:00';
  let ws: WebSocket;
  let reactionEvents: {
    type: 'thumbs' | 'applause' | 'smile';
    position: { x: number; y: number };
    timestamp: number;
  }[] = [];

  async function getStream() {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: true,
        audio: false
      });
      videoRef.srcObject = stream;
      error = null;
    } catch (err) {
      error = 'Unable to access camera. Please make sure you have granted permission.';
      console.error(err);
    } finally {
      isLoading = false;
    }
  }

  function updateElapsedTime() {
    if (!recordingStartTime) return;
    const elapsed = Date.now() - recordingStartTime;
    const minutes = Math.floor(elapsed / 60000).toString().padStart(2, '0');
    const seconds = Math.floor((elapsed % 60000) / 1000).toString().padStart(2, '0');
    elapsedTime = `${minutes}:${seconds}`;
  }

  function startRecording() {
    isRecording = true;
    recordingStartTime = Date.now();
    const timer = setInterval(updateElapsedTime, 1000);
    return () => clearInterval(timer);
  }

  function stopRecording() {
    isRecording = false;
    recordingStartTime = null;
    elapsedTime = '00:00';
  }

  async function toggleRecording() {
    if (!isRecording) {
      startRecording();
    } else {
      stopRecording();
    }
  }

  async function stopStream() {
    if (stream) {
      stream.getTracks().forEach(track => track.stop());
    }
    if (ws) {
      ws.close();
    }
  }

  onMount(() => {
    getStream();

    const chart = initChart(chartCanvas);
    ws = connectWebSocket((data) => {
      updateChart(chart, data.timeline);
      emojiCounts = data.emojis;

      if (data.reactionEvents) {
        reactionEvents = [...reactionEvents, ...data.reactionEvents];
        // Clean up old reactions
        const now = Date.now();
        reactionEvents = reactionEvents.filter(r => now - r.timestamp < 3000);
      }

      if (data.applauseIntensity > 0.8) {
        confetti({ 
          particleCount: 100, 
          spread: 70,
          colors: ['#FFD700', '#FFA500', '#FF6347']
        });
      }
    });

    const cleanupTimer = startRecording();
    return () => {
      cleanupTimer();
      stopStream();
    };
  });

  onDestroy(() => {
    stopStream();
  });
</script>

<style>
  .container {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: #1a1a1a;
  }

  .content {
    position: relative;
    height: 100%;
    max-width: 1920px;
    margin: 0 auto;
  }

  .video-container {
    position: relative;
    width: 100%;
    height: 100%;
    background: #000;
  }

  video {
    position: absolute;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 0;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.8);
    z-index: 10;
    gap: 1rem;
    color: white;
  }

  .controls {
    position: absolute;
    top: 1rem;
    right: 1rem;
    z-index: 2;
    display: flex;
    gap: 1rem;
  }

  .record-button {
    background: rgba(255, 255, 255, 0.9);
    border: none;
    border-radius: 2rem;
    padding: 0.75rem 1.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-weight: 600;
    transition: all 0.2s;
    backdrop-filter: blur(10px);
  }

  .record-button:hover {
    background: white;
    transform: translateY(-1px);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .record-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #ef4444;
    animation: pulse 2s infinite;
  }

  .timer {
    font-family: monospace;
    font-size: 1.1rem;
  }

  .chart-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: linear-gradient(to bottom, 
      rgba(0, 0, 0, 0.4) 0%,
      rgba(0, 0, 0, 0.2) 50%,
      rgba(0, 0, 0, 0.4) 100%
    );
    z-index: 1;
    pointer-events: none;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  canvas.chart {
    width: 100%;
    height: 70%;
    opacity: 0.8;
  }

  .stats-wrapper {
    position: absolute;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 2;
  }

  .error-container {
    position: absolute;
    top: 1rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 3;
    width: 90%;
    max-width: 600px;
  }

  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
  }

  @media (max-width: 768px) {
    .controls {
      top: auto;
      bottom: 1rem;
      right: 1rem;
    }

    canvas.chart {
      height: 60%;
    }
  }
</style>

<div class="container">
  <div class="content">
    <div class="video-container">
      <video bind:this={videoRef} autoplay muted playsinline></video>
      
      {#if isLoading}
        <div class="loading-overlay">
          <LoadingSpinner size="60px" color="#ffffff" />
          <span>Initializing camera...</span>
        </div>
      {/if}

      {#if error}
        <div class="error-container">
          <Alert message={error} type="error" />
        </div>
      {/if}

      <div class="controls">
        <button class="record-button" on:click={toggleRecording}>
          {#if isRecording}
            <div class="record-indicator"></div>
            <span class="timer">{elapsedTime}</span>
            <span>Stop</span>
          {:else}
            <span>Start Recording</span>
          {/if}
        </button>
      </div>

      <div class="chart-container">
        <canvas bind:this={chartCanvas} class="chart"></canvas>
      </div>
      
      <div class="stats-wrapper">
        <Stats counts={emojiCounts} />
      </div>

      <FloatingReactions reactions={reactionEvents} />
    </div>
  </div>
</div>
