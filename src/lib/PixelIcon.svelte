<script lang="ts">
    export let variant: "thinking" | "reading" | "responding" | "creating" = "thinking";
    export let size = 5;
    export let color: string | undefined = undefined;
    let className = "";
    export { className as class };
</script>

<span
    class="px {className}"
    data-variant={variant}
    style="--cell: {size}px;{color ? ` --c1: ${color}; --c2: ${color}; --glow: ${color};` : ''}"
    aria-hidden="true"
>
    {#each Array(4) as _}<i></i>{/each}
</span>

<style>
    .px {
        --c1: #6cc9b8;
        --c2: #a8e6dc;
        --glow: #6cc9b8;

        display: inline-grid;
        grid-template-columns: repeat(2, var(--cell));
        gap: 1px;
        color: var(--glow);
        filter: drop-shadow(0 0 1px color-mix(in srgb, currentColor 90%, transparent))
            drop-shadow(0 0 3px color-mix(in srgb, currentColor 70%, transparent))
            drop-shadow(0 0 7px color-mix(in srgb, currentColor 45%, transparent))
            drop-shadow(0 0 14px color-mix(in srgb, currentColor 22%, transparent));
    }
    i {
        width: var(--cell);
        height: var(--cell);
        border-radius: 1px;
    }

    i:nth-child(1) { --r: 0; --k: 0; --d: 0; background: var(--c1); }
    i:nth-child(2) { --r: 0; --k: 1; --d: 1; background: var(--c1); }
    i:nth-child(3) { --r: 1; --k: 0; --d: 1; background: var(--c2); }
    i:nth-child(4) { --r: 1; --k: 1; --d: 2; background: var(--c2); }

    .px[data-variant="thinking"] i {
        animation: tide 1.6s ease-in-out infinite;
        animation-delay: calc(var(--k) * -0.8s);
    }

    .px[data-variant="reading"] i {
        animation: scan 1.4s ease-in-out infinite;
        animation-delay: calc(var(--r) * -0.7s);
    }

    .px[data-variant="responding"] i {
        animation: ripple 1.3s ease-out infinite;
        animation-delay: calc(var(--d) * 0.18s);
    }

    .px[data-variant="creating"] i {
        animation: fill 2s ease-in-out infinite;
        animation-delay: calc((1 - var(--r)) * 0.25s);
    }

    @keyframes tide {
        0%, 100% { opacity: 0.4; transform: translateY(32%); }
        50%      { opacity: 1;   transform: translateY(-32%); }
    }
    @keyframes scan {
        0%, 100% { opacity: 0.18; }
        40%, 60% { opacity: 1; }
    }
    @keyframes ripple {
        0%   { opacity: 1;    transform: scale(1); }
        60%  { opacity: 0.25; transform: scale(0.7); }
        100% { opacity: 1;    transform: scale(1); }
    }
    @keyframes fill {
        0%, 100% { opacity: 0.12; }
        30%, 65% { opacity: 1; }
    }

    @media (prefers-reduced-motion: reduce) {
        i { animation-duration: 0.01s; }
    }
</style>
