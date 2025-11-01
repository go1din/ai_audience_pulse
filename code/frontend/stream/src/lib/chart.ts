import { Chart } from 'chart.js/auto';

export function initChart(canvas: HTMLCanvasElement): Chart {
  return new Chart(canvas, {
    type: 'line',
    data: {
      labels: [],
      datasets: [{
        label: 'Engagement',
        data: [],
        borderColor: 'rgba(255, 99, 132, 1)',
        tension: 0.4,
        fill: false
      }]
    },
    options: {
      animation: false,
      scales: {
        x: { display: false },
        y: { min: 0, max: 1 }
      }
    }
  });
}

export function updateChart(chart: Chart, timeline: number[]) {
  chart.data.labels = timeline.map((_, i) => i.toString());
  chart.data.datasets[0].data = timeline;

  const silence = timeline.every(v => v < 0.1);
  chart.data.datasets[0].borderColor = silence ? 'rgba(0, 150, 255, 1)' : 'rgba(255, 99, 132, 1)';

  chart.update();
}
