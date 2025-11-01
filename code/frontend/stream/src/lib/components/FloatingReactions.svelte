<script lang="ts">
  import { onMount } from 'svelte';
  
  export let reactions: {
    type: 'thumbs' | 'applause' | 'smile';
    position: { x: number; y: number };
    timestamp: number;
  }[] = [];

  const EMOJI_MAP = {
    thumbs: '👍',
    applause: '👏',
    smile: '😊'
  };

  const REACTION_LIFETIME = 3000; // 3 seconds
  const TIME_WINDOW = 1000; // 1 second window for reaction frequency
  const MIN_SIZE = 1; // Base size in rem
  const MAX_SIZE = 8; // Maximum size in rem
  const REACTION_THRESHOLD = 120; // Threshold for maximum size
  
  let visibleReactions = reactions;
  let reactionFrequency = 0;

  function calculateEmojiSize(timestamp: number): number {
    const now = Date.now();
    // Count reactions in the last second
    const recentReactions = reactions.filter(r => 
      r.timestamp > now - TIME_WINDOW && 
      r.timestamp <= now
    ).length;

    // Calculate size based on frequency
    const sizeRatio = Math.min(recentReactions / REACTION_THRESHOLD, 1);
    const size = MIN_SIZE + (MAX_SIZE - MIN_SIZE) * sizeRatio;
    
    return size;
  }

  $: {
    const now = Date.now();
    visibleReactions = reactions
      .filter(r => now - r.timestamp < REACTION_LIFETIME)
      .map(r => ({
        ...r,
        size: calculateEmojiSize(r.timestamp)
      }));
  }
</script>

<style>
  .reaction {
    position: absolute;
    pointer-events: none;
    opacity: 0;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
    will-change: transform, opacity;
    transition: font-size 0.3s ease-out;
  }

  @keyframes floatUp {
    0% {
      transform: translateY(0) scale(0.5);
      opacity: 0;
    }
    15% {
      transform: translateY(-15px) scale(1.2);
      opacity: 1;
    }
    85% {
      transform: translateY(-100px) scale(1);
      opacity: 1;
    }
    100% {
      transform: translateY(-120px) scale(0.8);
      opacity: 0;
    }
  }

  .reaction {
    animation: floatUp var(--duration) cubic-bezier(0.4, 0, 0.2, 1) forwards;
  }

  .thumbs { --duration: 2.8s; }
  .applause { --duration: 3.2s; }
  .smile { --duration: 3s; }

  /* Add size classes for different intensities */
  .reaction-intense {
    filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3));
  }

  .reaction-super-intense {
    filter: drop-shadow(0 6px 12px rgba(0, 0, 0, 0.4));
  }
</style>

{#each visibleReactions as reaction (reaction.timestamp)}
  <div 
    class="reaction {reaction.type} {reaction.size > 3 ? 'reaction-intense' : ''} {reaction.size > 3.5 ? 'reaction-super-intense' : ''}"
    style="
      left: {reaction.position.x}%; 
      top: {reaction.position.y}%;
      font-size: {reaction.size}rem;
    "
  >
    {EMOJI_MAP[reaction.type]}
  </div>
{/each}