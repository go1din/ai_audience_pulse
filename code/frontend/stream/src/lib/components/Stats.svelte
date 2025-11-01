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
    ['thumbs', 'applause', 'smile'].forEach((type, index) => {
      if (counts[type] !== prevCounts[type]) {
        countElements[index]?.classList.add('changed');
        setTimeout(() => {
          countElements[index]?.classList.remove('changed');
        }, 300);
      }
    });
    prevCounts = { ...counts };
  });
</script>

<style>
  .stats-container {
    display: flex;
    gap: 2.5rem;
    padding: 0.75rem 1.5rem;
  }

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    min-width: 60px;
  }

  .emoji {
    font-size: 1.5rem;
    opacity: 0.9;
    transition: all 0.2s ease;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
  }

  .count {
    font-size: 1.125rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.9);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.025em;
    transition: all 0.2s ease;
  }

  .count:global(.changed) {
    animation: countChange 0.3s ease-in-out;
  }

  .label {
    display: none;
  }

  .stat:hover .emoji {
    transform: scale(1.2);
  }

  @keyframes countChange {
    0% { transform: scale(1); }
    50% { transform: scale(1.2); }
    100% { transform: scale(1); }
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