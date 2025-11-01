import { Chart } from 'chart.js/auto';

interface ChartColors {
  border: string;
  gradientTop: string;
  gradientMiddle: string;
  gradientBottom: string;
}

const COLOR_SCHEMES = {
  cold: {
    border: 'rgba(52, 211, 153, 1)',
    gradientTop: 'rgba(52, 211, 153, 0.7)',
    gradientMiddle: 'rgba(110, 231, 183, 0.3)',
    gradientBottom: 'rgba(167, 243, 208, 0.05)'
  },
  warm: {
    border: 'rgba(251, 191, 36, 1)',
    gradientTop: 'rgba(251, 191, 36, 0.7)',
    gradientMiddle: 'rgba(252, 211, 77, 0.4)',
    gradientBottom: 'rgba(254, 243, 199, 0.1)'
  },
  hot: {
    border: 'rgba(251, 146, 60, 1)',
    gradientTop: 'rgba(251, 146, 60, 0.8)',
    gradientMiddle: 'rgba(253, 186, 116, 0.5)',
    gradientBottom: 'rgba(254, 215, 170, 0.15)'
  }
};

export function initChart(canvas: HTMLCanvasElement): Chart {
  // Create a vertical gradient for the fill
  const ctx = canvas.getContext('2d');
  let gradient: CanvasGradient | undefined;
  const colors = COLOR_SCHEMES.cold;
  if (ctx) {
    gradient = ctx.createLinearGradient(0, 0, 0, canvas.height);
    gradient.addColorStop(0, colors.gradientTop);
    gradient.addColorStop(0.5, colors.gradientMiddle);
    gradient.addColorStop(1, colors.gradientBottom);
  }

  return new Chart(canvas, {
    type: 'line',
    data: {
      labels: [],
      datasets: [{
        label: 'Live Hüllkurve',
        data: [],
        borderColor: colors.border,
        backgroundColor: gradient || 'rgba(100,120,255,0.2)',
        tension: 0.5,
        fill: true,
        pointRadius: 0,
  borderWidth: 3
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: {
        duration: 400,
        easing: 'easeOutCubic'
      },
      plugins: {
        legend: { display: false }
      },
      scales: {
        x: { 
          display: false,
          grid: {
            display: true,
            drawOnChartArea: true,
            drawTicks: false,
            color: 'rgba(52, 211, 153, 0.2)',
            lineWidth: 1.5,
            tickLength: 0
          }
        },
        y: { 
          min: 0, 
          max: 1,
          ticks: { 
            display: false,
            count: 6
          },
          grid: { 
            display: true,
            drawOnChartArea: true,
            drawTicks: false,
            color: 'rgba(52, 211, 153, 0.25)',
            lineWidth: 1.5
          } 
        }
      },
      elements: {
        line: {
          borderJoinStyle: 'round',
          borderCapStyle: 'round',
        }
      }
    }
  });
}

interface SilenceInfo {
  isSilent: boolean;
  position: { x: number; y: number };
  audioLevel: number;
}

function getColorForValue(value: number): string {
  // Return color based on audio level (0-1 range)
  if (value >= 0.6) {
    return 'rgba(251, 146, 60, 1)'; // Hot orange
  } else if (value >= 0.3) {
    return 'rgba(251, 191, 36, 1)'; // Warm yellow
  } else {
    return 'rgba(52, 211, 153, 1)'; // Cold green
  }
}

function getColorScheme(audioLevel: number): ChartColors {
  // Base color on actual audio envelope level (0-1 range)
  if (audioLevel >= 0.6) {
    return COLOR_SCHEMES.hot;
  } else if (audioLevel >= 0.3) {
    return COLOR_SCHEMES.warm;
  } else {
    return COLOR_SCHEMES.cold;
  }
}

export function updateChart(
  chart: Chart, 
  timeline: number[], 
  reactionIntensity: number = 0
): SilenceInfo {
  chart.data.labels = timeline.map((_, i) => i.toString());
  chart.data.datasets[0].data = timeline;

  // Calculate average audio level from recent timeline values
  const recentValues = timeline.slice(-5);
  const avgAudioLevel = recentValues.reduce((sum, val) => sum + val, 0) / recentValues.length;

  // Create a horizontal gradient that changes color based on data values
  const ctx = chart.ctx;
  if (ctx && timeline.length > 0) {
    // Create horizontal gradient across the chart
    const gradient = ctx.createLinearGradient(0, 0, chart.width, 0);
    
    // Add color stops based on each data point's value
    timeline.forEach((value, index) => {
      const position = index / (timeline.length - 1 || 1);
      const color = getColorForValue(value);
      
      // Create vertical gradient for each segment
      const segmentGradient = ctx.createLinearGradient(0, 0, 0, chart.height);
      
      if (value >= 0.6) {
        // Hot - orange
        gradient.addColorStop(position, 'rgba(251, 146, 60, 0.8)');
      } else if (value >= 0.3) {
        // Warm - yellow
        gradient.addColorStop(position, 'rgba(251, 191, 36, 0.7)');
      } else {
        // Cold - green
        gradient.addColorStop(position, 'rgba(52, 211, 153, 0.7)');
      }
    });
    
    chart.data.datasets[0].backgroundColor = gradient;
    
    // Set border color based on most recent value
    const lastValue = timeline[timeline.length - 1];
    chart.data.datasets[0].borderColor = getColorForValue(lastValue);
  }

  // Detect silence (if last 3 values are below threshold)
  const SILENCE_THRESHOLD = 0.1;
  const lastValues = timeline.slice(-3);
  const isSilent = lastValues.every(v => v < SILENCE_THRESHOLD);

  // Calculate position for silence indicator - left side, middle of window
  const position = {
    x: 0, // Left side (will be positioned with CSS)
    y: 0  // Middle (will be positioned with CSS)
  };

  chart.update('none'); // Use 'none' for smoother performance with frequent updates
  
  return { isSilent, position, audioLevel: avgAudioLevel };
}
