<script lang="ts">
  import { onMount } from 'svelte';
  import LoadingSpinner from './LoadingSpinner.svelte';
  import SkeletonLoader from './SkeletonLoader.svelte';
  import ErrorBoundary from './ErrorBoundary.svelte';
  import { handleAsyncError } from '../../lib/utils';
  
  export let promise: Promise<any>;
  export let loadingComponent = 'skeleton';
  export let errorComponent = true;
  export let skeletonCount = 1;
  export let skeletonHeight = '20px';
  export let loadingSize: 'small' | 'medium' | 'large' = 'medium';
  export let loadingColor = '#3b82f6';
  
  let loading = true;
  let error: Error | null = null;
  let data: any = null;
  
  onMount(async () => {
    const [result, err] = await handleAsyncError(promise);
    loading = false;
    
    if (err) {
      error = err;
    } else {
      data = result;
    }
  });
  
  function handleRetry() {
    loading = true;
    error = null;
    onMount(async () => {
      const [result, err] = await handleAsyncError(promise);
      loading = false;
      
      if (err) {
        error = err;
      } else {
        data = result;
      }
    });
  }
</script>

{#if loading}
  {#if loadingComponent === 'skeleton'}
    <SkeletonLoader count={skeletonCount} height={skeletonHeight} />
  {:else}
    <div class="loading-container">
      <LoadingSpinner size={loadingSize} color={loadingColor} />
    </div>
  {/if}
{:else if error}
  {#if errorComponent}
    <ErrorBoundary on:retry={handleRetry} />
  {:else}
    <div class="error-message">{error.message}</div>
  {/if}
{:else}
  <slot {data} />
{/if}

<style>
  .loading-container {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 20px;
  }
  
  .error-message {
    color: #ff5c8a;
    padding: 10px;
    background: #3a1a2c;
    border: 1px solid #ff5c8a;
    border-radius: 4px;
    font-size: 12px;
  }
</style>
