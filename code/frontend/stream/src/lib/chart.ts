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
      animation: {
        duration: 400,
        easing: 'easeOutCubic'
      },
      plugins: {
        legend: { display: false }
      },
      scales: {
        x: { display: false },
        y: { min: 0, max: 1, grid: { color: 'rgba(255,255,255,0.07)' } }
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

  // Update gradient fill and colors based on audio level
  const ctx = chart.ctx;
  if (ctx) {
    const colors = getColorScheme(avgAudioLevel);
    const gradient = ctx.createLinearGradient(0, 0, 0, chart.height);
    gradient.addColorStop(0, colors.gradientTop);
    gradient.addColorStop(0.5, colors.gradientMiddle);
    gradient.addColorStop(1, colors.gradientBottom);
    chart.data.datasets[0].backgroundColor = gradient;
    chart.data.datasets[0].borderColor = colors.border;
  }

  // Detect silence (if last 3 values are below threshold)
  const SILENCE_THRESHOLD = 0.1;
  const lastValues = timeline.slice(-3);
  const isSilent = lastValues.every(v => v < SILENCE_THRESHOLD);

  // Calculate position for silence indicator based on chart dimensions
  const position = {
    x: chart.width * 0.8, // Position towards the right
    y: chart.height * 0.3  // Position in upper third
  };

  chart.update('active');
  
  return { isSilent, position, audioLevel: avgAudioLevel };
}
