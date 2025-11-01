<script lang="ts">
  export let currentIntensity = 0;
  
  function getActiveState(min: number, max: number): boolean {
    return currentIntensity >= min && currentIntensity < max;
  }
  
  // Convert audio level (0-1) to percentage for display
  $: audioPercent = Math.round(currentIntensity * 100);
</script>

<style>
  .legend-container {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 0.75rem 1.5rem;
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(10px);
    border-radius: 1.5rem;
    border: 1px solid rgba(255, 255, 255, 0.15);
    font-size: 0.875rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.9);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.75rem;
    border-radius: 1rem;
    transition: all 0.3s ease;
    opacity: 0.6;
  }

  .legend-item.active {
    opacity: 1;
    background: rgba(255, 255, 255, 0.1);
    transform: scale(1.05);
  }

  .color-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    transition: all 0.3s ease;
  }

  .legend-item.active .color-dot {
    box-shadow: 0 0 12px currentColor;
    transform: scale(1.2);
  }

  .color-dot.green {
    background: rgba(52, 211, 153, 1);
  }

  .color-dot.yellow {
    background: rgba(251, 191, 36, 1);
  }

  .color-dot.orange {
    background: rgba(251, 146, 60, 1);
  }

  .legend-label {
    white-space: nowrap;
    letter-spacing: 0.025em;
  }

  .legend-range {
    font-size: 0.75rem;
    opacity: 0.7;
    margin-left: 0.25rem;
  }

  @media (max-width: 768px) {
    .legend-container {
      flex-direction: column;
      gap: 0.5rem;
      padding: 0.5rem 1rem;
    }

    .legend-item {
      width: 100%;
      justify-content: center;
    }
  }
</style>

<div class="legend-container">
  <div class="legend-item" class:active={getActiveState(0, 0.3)}>
    <div class="color-dot green"></div>
    <span class="legend-label">
      Calm
      <span class="legend-range">(&lt;30%)</span>
    </span>
  </div>
  
  <div class="legend-item" class:active={getActiveState(0.3, 0.6)}>
    <div class="color-dot yellow"></div>
    <span class="legend-label">
      Engaged
      <span class="legend-range">(30-60%)</span>
    </span>
  </div>
  
  <div class="legend-item" class:active={getActiveState(0.6, 1.1)}>
    <div class="color-dot orange"></div>
    <span class="legend-label">
      Excited
      <span class="legend-range">(&gt;60%)</span>
    </span>
  </div>
</div>
