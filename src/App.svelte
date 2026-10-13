<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let open = false;
    let time = new Date();

    onMount(() => {
        const interval = setInterval(() => {
            time = new Date();
        }, 1000);

        return () => clearInterval(interval);
    });

    $: timeString = time.toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
    });

    $: dateString = time.toLocaleDateString([], {
        weekday: "short",
        month: "short",
        day: "numeric",
    });

    const nextFrame = () => new Promise(requestAnimationFrame);

    async function toggle() {
        if (!open) {
            await invoke("resize_island", { width: 400, height: 120 });
            await nextFrame();
            await nextFrame();
            open = true;
        } else {
            open = false;
        }
    }

    function onEnd(e: TransitionEvent) {
        if (!open && e.propertyName === "height") {
            invoke("resize_island", { width: 260, height: 80 });
        }
    }
</script>

<main>
    <button
        class="island"
        class:open
        on:click={toggle}
    >
        <div class="content">
            <div class="compact-view" class:hidden={open}>
                <span class="clock">{timeString}</span>
            </div>

            <div class="expanded-view" class:visible={open}>
                <span class="large-time">{timeString}</span>
                <span class="date">{dateString}</span>
            </div>
        </div>
    </button>
</main>

<style>
    main {
        display: flex;
        justify-content: center;
    }
    .island {
        --ear: 14px;
        position: relative;
        width: 180px;
        height: 32px;
        border: 0;
        border-radius: 0 0 18px 18px;
        background: #000;
        color: #fff;
        cursor: pointer;
        padding: 0;
        will-change: width, height, border-radius;
        transform: translateZ(0);
        transition:
            height 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0s,
            width 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s,
            border-radius 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s;
    }
    .island::before,
    .island::after {
        content: "";
        position: absolute;
        top: 0;
        width: var(--ear);
        height: var(--ear);
    }
    .island::before {
        left: calc(var(--ear) * -1);
        background: radial-gradient(
            circle at 0 100%,
            transparent calc(var(--ear) - 0.5px),
            #000 var(--ear)
        );
    }
    .island::after {
        right: calc(var(--ear) * -1);
        background: radial-gradient(
            circle at 100% 100%,
            transparent calc(var(--ear) - 0.5px),
            #000 var(--ear)
        );
    }
    .island.open {
        width: 288px;
        height: 100px;
        border-radius: 0 0 28px 28px;
        transition:
            width 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0s,
            height 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s,
            border-radius 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s;
    }

    /* Content Layout & Smooth Transitions */
    .content {
        position: relative;
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: system-ui, -apple-system, sans-serif;
    }

    .compact-view {
        position: absolute;
        font-size: 13px;
        font-weight: 600;
        font-variant-numeric: tabular-nums;
        opacity: 1;
        transform: scale(1);
        transition:
            opacity 0.3s ease 0s,
            transform 0.3s ease 0s;
    }

    .compact-view.hidden {
        opacity: 0;
        transform: scale(0.85);
        pointer-events: none;
    }

    .expanded-view {
        position: absolute;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 2px;
        opacity: 0;
        transform: scale(0.9);
        pointer-events: none;
        transition:
            opacity 0.3s ease 0s,
            transform 0.3s ease 0s;
    }

    .expanded-view.visible {
        opacity: 1;
        transform: scale(1);
        pointer-events: auto;
        transition:
            opacity 0.25s ease 0.35s,
            transform 0.25s ease 0.35s;
    }

    .large-time {
        font-size: 24px;
        font-weight: 700;
        font-variant-numeric: tabular-nums;
    }

    .date {
        font-size: 11px;
        color: #8e8e93;
        text-transform: uppercase;
        letter-spacing: 0.8px;
    }
</style>