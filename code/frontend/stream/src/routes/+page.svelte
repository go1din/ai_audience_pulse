<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import confetti from 'canvas-confetti';
  import { initChart, updateChart } from '$lib/chart';
  import { connectWebSocket } from '$lib/websocket';
  import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';
  import Alert from '$lib/components/Alert.svelte';
  import Stats from '$lib/components/Stats.svelte';
  import FloatingReactions from '$lib/components/FloatingReactions.svelte';
  import SilenceIndicator from '$lib/components/SilenceIndicator.svelte';
  import ColorLegend from '$lib/components/ColorLegend.svelte';

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
  
  let silenceInfo = {
    isSilent: false,
    position: { x: 0, y: 0 },
    audioLevel: 0
  };
  let reactionIntensity = 0;

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
      // Calculate total reaction intensity
      reactionIntensity = data.emojis.thumbs + data.emojis.applause + data.emojis.smile;
      silenceInfo = updateChart(chart, data.timeline, reactionIntensity);
      emojiCounts = data.emojis;

      if (data.reactionEvents) {
        reactionEvents = [...reactionEvents, ...data.reactionEvents];
        // Clean up old reactions
        const now = Date.now();
        reactionEvents = reactionEvents.filter(r => now - r.timestamp < 3000);
      }

      // Only trigger confetti if recording and 4+ applause reactions
      if (isRecording && data.applauseIntensity === 1) {
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
  @import '../lib/styles/utils.css';

  .container {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 50%, #0f1729 100%);
    color: #f8fafc;
  }

  .container::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: 
      radial-gradient(circle at 20% 30%, rgba(52, 211, 153, 0.08) 0%, transparent 50%),
      radial-gradient(circle at 80% 70%, rgba(251, 191, 36, 0.06) 0%, transparent 50%);
    pointer-events: none;
    z-index: 0;
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
    box-shadow: 0 0 50px rgba(0, 0, 0, 0.5);
  }

  video {
    position: absolute;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 0;
    transition: filter 0.3s ease;
  }

  video:not(.recording) {
    filter: grayscale(0.3) contrast(1.1);
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
    background: rgba(15, 23, 42, 0.95);
    z-index: 10;
    gap: 1.5rem;
    color: #f8fafc;
    animation: fadeIn 0.5s ease-out;
  }

  .loading-text {
    font-size: 1.25rem;
    font-weight: 500;
    letter-spacing: 0.025em;
    opacity: 0.9;
  }

  .controls {
    position: absolute;
    top: 2rem;
    right: 2rem;
    z-index: 2;
    display: flex;
    gap: 1rem;
  }

  .record-button {
    background: rgba(255, 255, 255, 0.08);
    border: 1.5px solid rgba(255, 255, 255, 0.15);
    border-radius: 3rem;
    padding: 1rem 2.25rem;
    display: flex;
    align-items: center;
    gap: 0.875rem;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.9375rem;
    color: #fff;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    backdrop-filter: blur(16px) saturate(180%);
    letter-spacing: 0.05em;
    box-shadow: 
      0 8px 16px rgba(0, 0, 0, 0.2),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
    position: relative;
    overflow: hidden;
  }

  .record-button::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s;
  }

  .record-button:hover::before {
    left: 100%;
  }

  .record-button:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.25);
    transform: translateY(-3px);
    box-shadow: 
      0 12px 24px rgba(0, 0, 0, 0.25),
      inset 0 1px 0 rgba(255, 255, 255, 0.15);
  }

  .record-button:active {
    transform: translateY(-1px);
  }

  .record-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #ef4444;
    box-shadow: 
      0 0 12px rgba(239, 68, 68, 0.6),
      0 0 24px rgba(239, 68, 68, 0.3);
  }

  .record-indicator.active {
    animation: pulseGlow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  .timer {
    font-family: 'JetBrains Mono', monospace;
    font-size: 1.125rem;
    font-weight: 500;
    letter-spacing: 0.05em;
    color: rgba(255, 255, 255, 0.9);
  }

  @keyframes pulseGlow {
    0%, 100% {
      opacity: 1;
      box-shadow: 
        0 0 12px rgba(239, 68, 68, 0.6),
        0 0 24px rgba(239, 68, 68, 0.3);
      transform: scale(1);
    }
    50% {
      opacity: 0.7;
      box-shadow: 
        0 0 20px rgba(239, 68, 68, 0.9),
        0 0 40px rgba(239, 68, 68, 0.5);
      transform: scale(1.1);
    }
  }

  .chart-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: linear-gradient(180deg, 
      rgba(10, 14, 39, 0.3) 0%,
      rgba(10, 14, 39, 0.02) 30%,
      rgba(10, 14, 39, 0.02) 70%,
      rgba(10, 14, 39, 0.3) 100%
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
    opacity: 0.92;
    filter: drop-shadow(0 8px 24px rgba(0, 0, 0, 0.3));
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }

  canvas.chart:hover {
    opacity: 0.98;
  }

  .stats-wrapper {
    position: absolute;
    bottom: 2.5rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 2;
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    background: rgba(255, 255, 255, 0.06);
    backdrop-filter: blur(20px) saturate(180%);
    border-radius: 2.5rem;
    border: 1.5px solid rgba(255, 255, 255, 0.12);
    box-shadow: 
      0 8px 32px rgba(0, 0, 0, 0.3),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  .legend-wrapper {
    position: absolute;
    top: 2.5rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 2;
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    animation: slideDown 0.6s cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translate(-50%, -20px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }

  .stats-wrapper:hover {
    opacity: 1;
  }

  .video-container:not(:hover) .stats-wrapper {
    opacity: 0.7;
  }

  .error-container {
    position: absolute;
    top: 2rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 3;
    width: 90%;
    max-width: 600px;
    animation: slideDown 0.5s ease-out;
  }

  @keyframes slideDown {
    from {
      transform: translate(-50%, -20px);
      opacity: 0;
    }
    to {
      transform: translate(-50%, 0);
      opacity: 1;
    }
  }

  .status-badge {
    position: absolute;
    top: 2.5rem;
    left: 2.5rem;
    padding: 0.625rem 1.25rem;
    border-radius: 2rem;
    font-size: 0.875rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 0.625rem;
    z-index: 2;
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    backdrop-filter: blur(16px) saturate(180%);
    box-shadow: 
      0 4px 16px rgba(0, 0, 0, 0.2),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  .status-badge.recording {
    background: rgba(239, 68, 68, 0.15);
    color: #fef2f2;
    border: 1.5px solid rgba(239, 68, 68, 0.4);
    box-shadow: 
      0 4px 16px rgba(239, 68, 68, 0.3),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  .status-badge.standby {
    background: rgba(59, 130, 246, 0.15);
    color: #eff6ff;
    border: 1.5px solid rgba(59, 130, 246, 0.4);
    box-shadow: 
      0 4px 16px rgba(59, 130, 246, 0.3),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  @media (max-width: 768px) {
    .controls {
      top: auto;
      bottom: 1.5rem;
      right: 1.5rem;
    }

    .status-badge {
      top: 1.5rem;
      left: 1.5rem;
    }

    canvas.chart {
      height: 60%;
    }

    .stats-wrapper {
      bottom: 2rem;
    }
  }

  .chart-container,
  .stats-wrapper,
  .floating-reactions {
    transition: opacity 0.3s ease-out;
  }
  
  .hidden {
    opacity: 0;
    pointer-events: none;
  }

  @media (min-width: 1280px) {
    .controls {
      right: 3rem;
    }

    .status-badge {
      left: 3rem;
    }
  }
</style>

<div class="container">
  <div class="content">
    <div class="video-container">
      <video 
        bind:this={videoRef} 
        autoplay 
        muted 
        playsinline
        class:recording={isRecording}
      ></video>
      
      {#if isLoading}
        <div class="loading-overlay">
          <LoadingSpinner size="64px" color="#60a5fa" />
          <span class="loading-text">Initializing camera</span>
        </div>
      {/if}

      {#if error}
        <div class="error-container">
          <Alert message={error} type="error" />
        </div>
      {/if}

      <div class="status-badge glass-effect-dark {isRecording ? 'recording' : 'standby'}">
        <div class="record-indicator {isRecording ? 'active' : ''}"></div>
        <span>{isRecording ? 'Processing' : 'Ready'}</span>
      </div>

      <div class="controls">
        <button 
          class="record-button hover-scale" 
          on:click={toggleRecording}
          style="--hover-scale: 1.05"
        >
          {#if isRecording}
            <div class="record-indicator active"></div>
            <span class="timer">{elapsedTime}</span>
            <span>Stop</span>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <circle cx="12" cy="12" r="3"></circle>
            </svg>
            <span>Start</span>
          {/if}
        </button>
      </div>

      <div class="chart-container" class:hidden={!isRecording}>
        <canvas bind:this={chartCanvas} class="chart"></canvas>
        <SilenceIndicator 
          isVisible={silenceInfo.isSilent && isRecording}
          x={silenceInfo.position.x}
          y={silenceInfo.position.y}
        />
      </div>
      
      <div class="legend-wrapper" class:hidden={!isRecording}>
        <ColorLegend currentIntensity={silenceInfo.audioLevel} />
      </div>

      <div class="stats-wrapper" class:hidden={!isRecording}>
        <Stats counts={emojiCounts} />
      </div>

      <div class="floating-reactions" class:hidden={!isRecording}>
        <FloatingReactions reactions={reactionEvents} />
      </div>
    </div>
  </div>
</div>
