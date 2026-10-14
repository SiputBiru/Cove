<script lang="ts">
    import "./app.css";
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
            await invoke("resize_island", { width: 310, height: 120 });
            await nextFrame();
            await nextFrame();
            open = true;
        } else {
            open = false;
        }
    }

    function onEnd(e: TransitionEvent) {
        if (!open && e.propertyName === "height") {
            invoke("resize_island", { width: 310, height: 32 });
        }
    }
</script>

<main class="flex justify-center">
    <button
        class="island relative border-0 bg-black text-white cursor-pointer p-0 will-change-[width,height,border-radius] transform:translateZ(0)"
        class:open
        on:click={toggle}
        on:transitionend={onEnd}
    >
        <div
            class="content relative w-full h-full flex items-center justify-center font-sans"
        >
            <div
                class="compact-view absolute text-[13px] font-semibold tabular-nums"
                class:hidden={open}
            >
                <span class="clock">{timeString}</span>
            </div>

            <div
                class="expanded-view flex flex-row items-center gap-0.5 overflow-x-scroll snap-x snap-mandatory hiddenScrollBar"
                class:visible={open}
            >
                <div class="flex flex-col w-full flex-[1_0_auto]">
                    <span class="large-time text-2xl font-bold tabular-nums"
                        >{timeString}</span
                    >
                    <span
                        class="date text-[11px] text-[#8e8e93] uppercase tracking-[0.8px]"
                        >{dateString}</span
                    >
                </div>
                <div class="flex flex-col w-full flex-[1_0_auto]">
                    <span class="large-time text-2xl font-bold tabular-nums"
                        >Todo Tasks</span
                    >
                    <span
                        class="date text-[11px] text-[#8e8e93] uppercase tracking-[0.8px]"
                        >Example</span
                    >
                </div>
            </div>
        </div>
    </button>
</main>

<style>
    .island {
        --ear: 14px;
        width: 180px;
        height: 32px;
        border-radius: 0 0 18px 18px;
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

    .compact-view {
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
        -webkit-overflow-scrolling: touch;
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

    .hiddenScrollBar {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }

    .hiddenScrollBar:global(::-webkit-scrollbar) {
        display: none;
    }
</style>
