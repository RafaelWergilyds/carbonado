<script lang="ts">
  let { size = 24 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 64 64" aria-hidden="true" data-tauri-drag-region>
  <g stroke="#a4a8b0" stroke-opacity="0.4" stroke-width="0.7" stroke-linejoin="round">
    <polygon points="48,8 21,8 26,17" fill="#6e7179" />
    <polygon points="48,8 26,17 48,17" fill="#4a4d53" />
    <polygon points="21,8 7,32 17,32" fill="#7a7d85" />
    <polygon points="21,8 17,32 26,17" fill="#55585f" />
    <polygon points="7,32 21,56 17,32" fill="#3a3c41" />
    <polygon points="21,56 26,47 17,32" fill="#26272b" />
    <polygon points="21,56 48,56 26,47" fill="#26272b" />
    <polygon points="48,56 48,47 26,47" fill="#141517" />
  </g>
  <polygon
    points="48,8 21,8 7,32 21,56 48,56 48,47 26,47 17,32 26,17 48,17"
    fill="none"
    stroke="#c3c6cc"
    stroke-opacity="0.5"
    stroke-width="1.1"
    stroke-linejoin="round"
  />
  <g transform="translate(35 32.5) scale(1.15)">
    <polygon points="-3,-5 3,-5 6,-1.5 -6,-1.5" fill="#8a8d95" />
    <polygon points="-6,-1.5 0,-1.5 0,6" fill="#55585f" />
    <polygon points="0,-1.5 6,-1.5 0,6" fill="#2a2b2f" />
    <polygon
      points="-3,-5 3,-5 6,-1.5 0,6 -6,-1.5"
      fill="none"
      stroke="#c3c6cc"
      stroke-opacity="0.55"
      stroke-width="0.7"
      stroke-linejoin="round"
    />
  </g>
</svg>
