<script lang="ts">
  export let size = '40px';
  export let color = '#3b82f6';
</script>

<style>
  .spinner {
    display: inline-block;
    border: 4px solid rgba(0, 0, 0, 0.1);
    border-left-color: var(--color);
    border-radius: 50%;
    width: var(--size);
    height: var(--size);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>

<div 
  class="spinner" 
  style="--size: {size}; --color: {color};"
  aria-label="Loading..."
  role="status"
></div>]]>