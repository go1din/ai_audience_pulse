<script lang="ts">
  export let counts: { thumbs: number; applause: number; smile: number };
  
  const reactions = [
    { emoji: '👍', count: counts.thumbs, label: 'Thumbs Up' },
    { emoji: '👏', count: counts.applause, label: 'Applause' },
    { emoji: '😊', count: counts.smile, label: 'Smiles' }
  ];
</script>

<style>
  .stats-container {
    display: flex;
    gap: 2rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.9);
    backdrop-filter: blur(10px);
    border-radius: 1rem;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 80px;
  }

  .emoji {
    font-size: 2rem;
    margin-bottom: 0.5rem;
    transition: transform 0.2s;
  }

  .count {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1a1a1a;
  }

  .label {
    font-size: 0.875rem;
    color: #666;
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