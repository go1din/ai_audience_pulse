<script lang="ts">
  export let counts: { thumbs: number; applause: number; smile: number };
  
  import { afterUpdate } from 'svelte';
  
  let prevCounts = { ...counts };
  
  // Make reactions reactive to changes in counts
  $: reactions = [
    { emoji: '👍', count: counts.thumbs, label: 'Thumbs Up' },
    { emoji: '👏', count: counts.applause, label: 'Applause' },
    { emoji: '😊', count: counts.smile, label: 'Smiles' }
  ];

  // Animate count changes
  afterUpdate(() => {
    const countElements = document.querySelectorAll('.count');
    const types: Array<keyof typeof counts> = ['thumbs', 'applause', 'smile'];
    types.forEach((type, index) => {
      if (counts[type] !== prevCounts[type]) {
        countElements[index]?.classList.add('changed');
        setTimeout(() => {
          countElements[index]?.classList.remove('changed');
        }, 400);
      }
    });
    prevCounts = { ...counts };
  });
</script>

<style>
  .stats-container {
    display: flex;
    gap: 2rem;
    padding: 0.875rem 1.75rem;
  }

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.625rem;
    min-width: 65px;
    position: relative;
    padding: 0.375rem 0.625rem;
    border-radius: 1rem;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .stat::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 1rem;
    background: rgba(255, 255, 255, 0.05);
    opacity: 0;
    transition: opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .stat:hover::before {
    opacity: 1;
  }

  .emoji {
    font-size: 1.625rem;
    opacity: 0.95;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    filter: drop-shadow(0 3px 6px rgba(0, 0, 0, 0.3));
    position: relative;
    z-index: 1;
  }

  .count {
    font-size: 1.25rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.95);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.05em;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    position: relative;
    z-index: 1;
  }

  .count:global(.changed) {
    animation: countChange 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .label {
    display: none;
  }

  .stat:hover {
    transform: translateY(-2px);
  }

  .stat:hover .emoji {
    transform: scale(1.15) rotate(-5deg);
  }

  .stat:hover .count {
    color: rgba(255, 255, 255, 1);
  }

  @keyframes countChange {
    0% { 
      transform: scale(1);
      color: rgba(255, 255, 255, 0.95);
    }
    50% { 
      transform: scale(1.3);
      color: rgba(251, 191, 36, 1);
      text-shadow: 0 0 12px rgba(251, 191, 36, 0.6);
    }
    100% { 
      transform: scale(1);
      color: rgba(255, 255, 255, 0.95);
    }
  }

  @media (max-width: 768px) {
    .stats-container {
      gap: 1.5rem;
      padding: 0.75rem 1.25rem;
    }

    .emoji {
      font-size: 1.375rem;
    }

    .count {
      font-size: 1.125rem;
    }
  }
</style>

<div class="stats-container">
  {#each reactions as { emoji, count, label }}
    <div class="stat">
      <span class="emoji">{emoji}</span>
      <span class="count">{count}</span>
      <span class="label">{label}</span>
    </div>
  {/each}
</div>