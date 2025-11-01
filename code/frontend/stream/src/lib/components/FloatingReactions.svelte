<script lang="ts">
  import { onMount } from 'svelte';
  
  export let reactions: {
    type: 'thumbs' | 'applause' | 'smile';
    position: { x: number; y: number };
    timestamp: number;
  }[] = [];
  export let isRecording: boolean = false;

  const EMOJI_MAP = {
    thumbs: '👍',
    applause: '👏',
    smile: '😊'
  };

  const REACTION_LIFETIME = 3000; // 3 seconds

  let visibleReactions = reactions;

  $: {
    const now = Date.now();
    visibleReactions = reactions.filter(r => now - r.timestamp < REACTION_LIFETIME);
  }
</script>

<style>
  .reaction {
    position: absolute;
    font-size: 2rem;
    pointer-events: none;
    animation: floatUp 3s ease-out forwards;
    opacity: 0;
  }

  @keyframes floatUp {
    0% {
      transform: translateY(0) scale(0.5);
      opacity: 0;
    }
    10% {
      transform: translateY(-10px) scale(1.2);
      opacity: 1;
    }
    90% {
      transform: translateY(-100px) scale(1);
      opacity: 1;
    }
    100% {
      transform: translateY(-120px) scale(0.8);
      opacity: 0;
    }
  }

  .thumbs { animation-duration: 2.8s; }
  .applause { animation-duration: 3.2s; }
  .smile { animation-duration: 3s; }
</style>

{#each visibleReactions as reaction (reaction.timestamp)}
  <div 
    class="reaction {reaction.type}"
    style="left: {reaction.position.x}%; top: {reaction.position.y}%;"
  >
    {EMOJI_MAP[reaction.type]}
  </div>
{/each}