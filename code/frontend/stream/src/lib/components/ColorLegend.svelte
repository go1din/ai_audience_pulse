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
    gap: 0.5rem;
    align-items: center;
    padding: 0.625rem 1rem;
    background: rgba(255, 255, 255, 0.06);
    backdrop-filter: blur(20px) saturate(180%);
    border-radius: 2rem;
    border: 1.5px solid rgba(255, 255, 255, 0.12);
    font-size: 0.8125rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.95);
    box-shadow: 
      0 8px 32px rgba(0, 0, 0, 0.3),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.875rem;
    border-radius: 1.25rem;
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    opacity: 0.5;
    position: relative;
  }

  .legend-item::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 1.25rem;
    background: rgba(255, 255, 255, 0.05);
    opacity: 0;
    transition: opacity 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .legend-item.active::before {
    opacity: 1;
  }

  .legend-item.active {
    opacity: 1;
    transform: scale(1.08);
    z-index: 1;
  }

  .color-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
  }

  .color-dot::after {
    content: '';
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    background: radial-gradient(circle, currentColor 0%, transparent 70%);
    opacity: 0;
    transition: opacity 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .legend-item.active .color-dot {
    box-shadow: 
      0 0 16px currentColor,
      0 0 32px currentColor,
      0 2px 8px rgba(0, 0, 0, 0.3);
    transform: scale(1.15);
  }

  .legend-item.active .color-dot::after {
    opacity: 0.4;
  }

  .color-dot.green {
    background: rgba(52, 211, 153, 1);
    color: rgba(52, 211, 153, 0.6);
  }

  .color-dot.yellow {
    background: rgba(251, 191, 36, 1);
    color: rgba(251, 191, 36, 0.6);
  }

  .color-dot.orange {
    background: rgba(251, 146, 60, 1);
    color: rgba(251, 146, 60, 0.6);
  }

  .legend-label {
    white-space: nowrap;
    letter-spacing: 0.05em;
    font-size: 0.8125rem;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .legend-range {
    font-size: 0.6875rem;
    opacity: 0.75;
    margin-left: 0.25rem;
    font-weight: 500;
  }

  @media (max-width: 768px) {
    .legend-container {
      flex-direction: column;
      gap: 0.375rem;
      padding: 0.5rem 0.875rem;
    }

    .legend-item {
      width: 100%;
      justify-content: center;
      padding: 0.375rem 0.75rem;
    }

    .legend-label {
      font-size: 0.75rem;
    }

    .legend-range {
      font-size: 0.625rem;
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
