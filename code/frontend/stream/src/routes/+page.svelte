<script>
  let stream = $state()
  let videoRef = $state()

  async function getStream() {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: true,
        audio: false
      })
      videoRef.srcObject = stream
    } catch (err) {
      console.error(err)
    }
  }

  async function stopStream() {
    stream.getTracks().forEach((/** @type {{ stop: () => any; }} */ track) => track.stop())
    videoRef.srcObject = null
  }

  $effect(() => {
    getStream()
    return () => {
      if (stream) {
        stopStream()
      }
    }
  })
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video bind:this={videoRef} autoplay > </video>
