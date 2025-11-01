// Theme handling - inverted for overlay (white text)
function theme() {
  return {
    grid: "rgba(255,255,255,0.2)",
    tick: "rgba(255,255,255,0.95)",
    line: "#7dd3fc",
    fill: "rgba(125,211,252,0.2)",
    point: "#bae6fd",
    emojiShadow: "rgba(0,0,0,0.5)",
  };
}

let colors = theme();

// Time-based data generation (seconds format, max 5 minutes = 300 seconds)
const MAX_SECONDS = 300; // 5 minutes max
const DATA_UPDATE_INTERVAL = 1; // Add data every second

// Format seconds to MM:SS
function formatTime(seconds) {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
}

// Count gestures from the gesture-text element
function getGestureCount() {
  const gestureText = document.getElementById('gesture-text');
  if (!gestureText || !gestureText.textContent) return 0;
  
  // Count emojis in the text
  const emojis = gestureText.textContent.trim().split(' ').filter(e => e.length > 0);
  return emojis.length;
}

// Count thumbs up specifically
function getThumbsUpCount() {
  const gestureText = document.getElementById('gesture-text');
  if (!gestureText || !gestureText.textContent) return 0;
  
  const emojis = gestureText.textContent.trim().split(' ').filter(e => e.length > 0);
  return emojis.filter(e => e === '👍').length;
}

let currentLabels = [];
let currentData = [];
let currentSeconds = 0;
let dataTimer = null;

// Track gesture events with emojis and their counts
const gestureEvents = new Map(); // Map of second -> { emoji, count }

// Function to add clap emoji when noise threshold is exceeded
window.triggerClapEmoji = function(peakLevel) {
  // Calculate intensity based on peak level (1-10 scale)
  const intensity = Math.min(Math.ceil((peakLevel - 80) / 4), 10); // 80% = 1, 100% = 5, beyond = up to 10
  
  // Determine emoji based on audio scores - use laughter if laughter_score > clapping_score AND > 0.4
  let selectedEmoji = '👏';
  if (window.audioScores && 
      window.audioScores.laughter_score > 0.4 && 
      window.audioScores.laughter_score > window.audioScores.clapping_score) {
    selectedEmoji = '😂';
  }
  
  // Add emoji at current second with intensity
  gestureEvents.set(currentSeconds, { emoji: selectedEmoji, count: intensity, peakLevel: peakLevel });
  
  // Also increase the chart value by intensity (so chart goes up with clapping)
  if (currentData.length > 0) {
    const currentIndex = currentData.length - 1;
    currentData[currentIndex] = Math.min(currentData[currentIndex] + intensity, 10);
  }
  
  // Display emoji in gesture text
  const gestureText = document.getElementById('gesture-text');
  if (gestureText) {
    const emojis = (selectedEmoji + ' ').repeat(Math.min(intensity, 5)); // Show 1-5 emojis
    gestureText.textContent = emojis.trim();
    
    // Clear after 3 seconds
    setTimeout(() => {
      if (gestureText.textContent.includes(selectedEmoji)) {
        gestureText.textContent = '';
      }
    }, 3000);
  }
  
  console.log(`${selectedEmoji} ${selectedEmoji === '�' ? 'Laughter' : 'Clap'}! Peak: ${peakLevel}%`);
};

// Dynamically update emoji map based on gesture events
function updateEmojiMap() {
  emojiMap.clear();
  emojiIndices.clear();
  
  currentLabels.forEach((label, index) => {
    // Parse MM:SS to seconds
    const [mins, secs] = label.split(':').map(Number);
    const totalSeconds = mins * 60 + secs;
    
    if (gestureEvents.has(totalSeconds)) {
      const eventData = gestureEvents.get(totalSeconds);
      emojiMap.set(index, eventData);
      emojiIndices.add(index);
    }
  });
}

const emojiMap = new Map();
const emojiIndices = new Set();

const emojiPointRadius = 3;

function getEmojiColor(index) {
  updateEmojiMap();
  
  // Find emoji indices by type
  const thumbsUpIndices = Array.from(emojiMap.entries())
    .filter(([idx, data]) => data.emoji === "👍")
    .map(([idx]) => idx);
  const thumbsDownIndices = Array.from(emojiMap.entries())
    .filter(([idx, data]) => data.emoji === "👎")
    .map(([idx]) => idx);
  const clapIndices = Array.from(emojiMap.entries())
    .filter(([idx, data]) => data.emoji === "👏")
    .map(([idx]) => idx);
  
  // Check if we're around thumbs down (red)
  for (const emojiIndex of thumbsDownIndices) {
    if (Math.abs(index - emojiIndex) <= emojiPointRadius) {
      return '#ff0000';
    }
  }
  
  // Check if we're around clap (yellow/orange)
  for (const emojiIndex of clapIndices) {
    if (Math.abs(index - emojiIndex) <= emojiPointRadius) {
      return '#ffa500';
    }
  }
  
  // Check if we're around thumbs up (green)
  for (const emojiIndex of thumbsUpIndices) {
    if (Math.abs(index - emojiIndex) <= emojiPointRadius) {
      return '#00ff00';
    }
  }
  
  return null;
}

// Plugin to draw emoji overlays
const emojiOverlay = {
  id: "emojiOverlay",
  afterDatasetsDraw(chart, args, pluginOpts) {
    const {ctx, chartArea, scales} = chart;
    const meta = chart.getDatasetMeta(0);
    ctx.save();

    ctx.textAlign = "center";
    ctx.textBaseline = "alphabetic";

    const baseFontPx = 22 * (window.devicePixelRatio || 1);
    
    const emojiOffset = 12;
    const emojiPadding = 15;

    for (const [idx, eventData] of emojiMap.entries()) {
      const pt = meta.data[idx];
      if (!pt) continue;
      
      const emoji = eventData.emoji;
      const count = eventData.count || 1;
      
      // Scale emoji size based on count and type
      let scaleFactor = 1;
      if (emoji === '👍') {
        scaleFactor = Math.min(1 + (count - 1) * 0.3, 2.5);
      } else if (emoji === '👏') {
        // Claps scale based on loudness (peakLevel stored in eventData)
        const peakLevel = eventData.peakLevel || 80;
        const loudnessScale = Math.min((peakLevel - 80) / 20, 1); // 80% = 0, 100% = 1
        scaleFactor = 1 + loudnessScale * 1.5; // Scale from 1x to 2.5x based on loudness
      }
      
      const fontPx = baseFontPx * scaleFactor;
      ctx.font = `${fontPx}px system-ui, Apple Color Emoji, Segoe UI Emoji, Noto Color Emoji`;
      
      const emojiSize = fontPx * 0.8;
      
      let {x, y} = pt.getProps(["x","y"], true);
      
      let emojiY = y - emojiOffset - (emojiSize - baseFontPx * 0.8);
      
      if (emojiY - emojiSize < chartArea.top + emojiPadding) {
        emojiY = chartArea.top + emojiPadding + emojiSize;
      }
      
      if (emojiY > chartArea.bottom - emojiPadding) {
        emojiY = chartArea.bottom - emojiPadding;
      }
      
      if (x < chartArea.left + emojiPadding) {
        x = chartArea.left + emojiPadding;
      }
      
      if (x > chartArea.right - emojiPadding) {
        x = chartArea.right - emojiPadding;
      }
      
      ctx.shadowColor = colors.emojiShadow;
      ctx.shadowBlur = 8;

      ctx.fillText(emoji, x, emojiY);
    }

    ctx.restore();
  }
};

function getPointColors() {
  return currentData.map((_, index) => {
    const emojiColor = getEmojiColor(index);
    return emojiColor || colors.point;
  });
}

function getBorderColors() {
  return currentData.map((_, index) => {
    const emojiColor = getEmojiColor(index);
    return emojiColor || colors.line;
  });
}

// Create chart
const cfg = {
  type: "line",
  data: {
    labels: currentLabels,
    datasets: [{
      label: "Hand Count",
      data: currentData,
      borderColor: colors.line,
      backgroundColor: colors.fill,
      pointBackgroundColor: colors.point,
      pointRadius: 0,
      pointHoverRadius: 3,
      tension: 0.4,
      fill: true,
      segment: {
        borderColor: (ctx) => {
          const index = ctx.p1DataIndex;
          const color1 = getEmojiColor(index);
          const color2 = getEmojiColor(index - 1);
          return color1 || color2 || colors.line;
        },
        backgroundColor: (ctx) => {
          const index = ctx.p1DataIndex;
          const color1 = getEmojiColor(index);
          const color2 = getEmojiColor(index - 1);
          if (color1 === '#00ff00' || color2 === '#00ff00') {
            return 'rgba(0, 255, 0, 0.2)';
          }
          if (color1 === '#ff0000' || color2 === '#ff0000') {
            return 'rgba(255, 0, 0, 0.2)';
          }
          if (color1 === '#ffa500' || color2 === '#ffa500') {
            return 'rgba(255, 165, 0, 0.2)';
          }
          return colors.fill;
        }
      }
    }]
  },
  options: {
    responsive: true,
    maintainAspectRatio: true,
    plugins: {
      legend: { display: false },
      tooltip: { mode: "index", intersect: false }
    },
    interaction: { mode: "nearest", intersect: false },
    scales: {
      x: {
        grid: { color: colors.grid },
        ticks: { 
          color: colors.tick,
          maxTicksLimit: 8
        }
      },
      y: {
        grid: { color: colors.grid },
        ticks: { color: colors.tick },
        min: 0,
        max: 10,
        title: {
          display: true,
          text: 'Hands Detected',
          color: colors.tick
        }
      }
    }
  },
  plugins: [emojiOverlay]
};

const ctx = document.getElementById("c");
let chart = new Chart(ctx, cfg);

// Track previous gesture text to detect emoji additions
let previousGestureText = '';
let previousThumbsUpCount = 0;

// Function to detect and record gesture emojis
function recordGestureEmoji() {
  const currentGestureText = document.getElementById('gesture-text').textContent;
  const currentThumbsUpCount = getThumbsUpCount();
  
  // Check if gesture text changed and contains emojis
  if (currentGestureText !== previousGestureText && currentGestureText.trim()) {
    const emojis = currentGestureText.trim().split(' ').filter(e => e.length > 0);
    
    // Record the first emoji at current time with count
    if (emojis.length > 0) {
      const firstEmoji = emojis[0];
      const count = firstEmoji === '👍' ? currentThumbsUpCount : emojis.filter(e => e === firstEmoji).length;
      
      // Update or create gesture event
      gestureEvents.set(currentSeconds, { emoji: firstEmoji, count: count });
    }
  } else if (currentThumbsUpCount !== previousThumbsUpCount && currentThumbsUpCount > 0) {
    // Update count if thumbs up count changed
    if (gestureEvents.has(currentSeconds)) {
      const eventData = gestureEvents.get(currentSeconds);
      if (eventData.emoji === '👍') {
        eventData.count = currentThumbsUpCount;
      }
    } else {
      gestureEvents.set(currentSeconds, { emoji: '👍', count: currentThumbsUpCount });
    }
  }
  
  previousGestureText = currentGestureText;
  previousThumbsUpCount = currentThumbsUpCount;
}

// Function to continuously add data points in real-time
function startDataAnimation() {
  currentLabels = [];
  currentData = [];
  currentSeconds = 0;
  gestureEvents.clear();
  chart.data.labels = currentLabels;
  chart.data.datasets[0].data = currentData;
  chart.data.datasets[0].pointBackgroundColor = colors.point;
  chart.data.datasets[0].borderColor = colors.line;
  chart.update('none');
  
  if (dataTimer) {
    clearInterval(dataTimer);
  }
  
  dataTimer = setInterval(() => {
    if (currentSeconds < MAX_SECONDS) {
      const label = formatTime(currentSeconds);
      const value = getGestureCount();
      
      // Record gesture emojis
      recordGestureEmoji();
      
      currentLabels.push(label);
      currentData.push(value);
      
      chart.data.labels = [...currentLabels];
      chart.data.datasets[0].data = [...currentData];
      
      updateEmojiMap();
      
      chart.data.datasets[0].pointBackgroundColor = getPointColors();
      chart.data.datasets[0].borderColor = getBorderColors();
      
      chart.update('active');
      
      // Update time display
      const totalTime = formatTime(MAX_SECONDS);
      const currentTime = formatTime(currentSeconds);
      document.querySelectorAll('.video-time').forEach(el => {
        el.textContent = `${currentTime} / ${totalTime}`;
      });
      
      // Update progress bar
      const progress = (currentSeconds / MAX_SECONDS) * 100;
      document.querySelectorAll('.video-progress-bar').forEach(el => {
        el.style.width = `${progress}%`;
      });
      
      currentSeconds++;
    }
  }, 1000);
}

function stopDataAnimation() {
  if (dataTimer) {
    clearInterval(dataTimer);
    dataTimer = null;
  }
  currentLabels = [];
  currentData = [];
  currentSeconds = 0;
  gestureEvents.clear();
  chart.data.labels = currentLabels;
  chart.data.datasets[0].data = currentData;
  chart.data.datasets[0].pointBackgroundColor = colors.point;
  chart.data.datasets[0].borderColor = colors.line;
  chart.update('none');
  
  // Reset time display
  document.querySelectorAll('.video-time').forEach(el => {
    el.textContent = '0:00 / 5:00';
  });
  
  // Reset progress bar
  document.querySelectorAll('.video-progress-bar').forEach(el => {
    el.style.width = '0%';
  });
}

// Export functions for use in index.js
window.timelineControls = {
  start: startDataAnimation,
  stop: stopDataAnimation
};
