import { Chart } from 'chart.js/auto';

export function initChart(canvas: HTMLCanvasElement): Chart {
  // Create a vertical gradient for the fill
  const ctx = canvas.getContext('2d');
  let gradient: CanvasGradient | undefined;
  if (ctx) {
    gradient = ctx.createLinearGradient(0, 0, 0, canvas.height);
    gradient.addColorStop(0, 'rgba(0, 212, 255, 0.7)');
    gradient.addColorStop(0.5, 'rgba(0, 255, 128, 0.3)');
    gradient.addColorStop(1, 'rgba(255, 255, 255, 0.05)');
  }

  return new Chart(canvas, {
    type: 'line',
    data: {
      labels: [],
      datasets: [{
        label: 'Live Hüllkurve',
        data: [],
        borderColor: 'rgba(0, 212, 255, 1)',
        backgroundColor: gradient || 'rgba(0,212,255,0.2)',
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
}

export function updateChart(chart: Chart, timeline: number[]): SilenceInfo {
  chart.data.labels = timeline.map((_, i) => i.toString());
  chart.data.datasets[0].data = timeline;

  // Optionally, update gradient fill if canvas size changes
  const ctx = chart.ctx;
  if (ctx) {
    const gradient = ctx.createLinearGradient(0, 0, 0, chart.height);
    gradient.addColorStop(0, 'rgba(0, 212, 255, 0.7)');
    gradient.addColorStop(0.5, 'rgba(0, 255, 128, 0.3)');
    gradient.addColorStop(1, 'rgba(255, 255, 255, 0.05)');
    chart.data.datasets[0].backgroundColor = gradient;
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
  
  return { isSilent, position };
}
