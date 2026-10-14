<script lang="ts">
    import "./app.css";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    const COMPACT = { width: 460, height: 32 };
    const OPEN = { width: 460, height: 160 };

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
            await invoke("resize_island", OPEN);
            await nextFrame();
            await nextFrame();
            open = true;
        } else {
            open = false;
        }
    }

    function onIslandTransitionEnd(e: TransitionEvent) {
        if (e.propertyName !== "width" || open) return;
        invoke("resize_island", COMPACT);
    }
</script>

<main class="flex justify-center">
    <button
        class="island relative border-0 bg-black text-white cursor-pointer p-0 will-change-[width,height,border-radius] transform:translateZ(0)"
        class:open
        on:click={toggle}
        on:transitionend={onIslandTransitionEnd}
    >
        <div
            class="content relative w-full h-full flex items-center justify-center font-sans"
        >
            <div
                class="compact-view w-full px-5 flex justify-between absolute text-[13px] font-semibold tabular-nums"
                class:is-hidden={open}
            >
                <div class="flex justify-center items-center ml-0.5">
                    <div
                        class="absolute flex size-2.5 bg-blue-100 rounded-full"
                    ></div>
                    <div
                        class="absolute flex size-3 bg-blue-100 rounded-full animate-ping"
                    ></div>
                </div>
                <div class="clock">{timeString}</div>
            </div>

            <div
                class="expanded-view w-full flex flex-row items-center gap-0.5 overflow-x-auto snap-x snap-mandatory"
                class:visible={open}
            >
                <div class="expand-child">
                    <span class="large-time text-2xl font-bold tabular-nums"
                        >{timeString}</span
                    >
                    <span
                        class="date text-[11px] text-[#8e8e93] uppercase tracking-[0.8px]"
                        >{dateString}</span
                    >
                </div>
                <div class="expand-child">
                    <span class="large-time text-2xl font-bold tabular-nums"
                        >Todo Tasks</span
                    >
                    <span
                        class="date text-[11px] text-[#8e8e93] uppercase tracking-[0.8px]"
                        >Example</span
                    >
                </div>
                <div class="expand-child">
                    <span class="large-time text-2xl font-bold tabular-nums"
                        >Pomodoro</span
                    >
                    <span
                        class="bg-blue-200 rounded-full w-fit py-0.75 px-5 mt-1.5 mx-auto text-black ring-5 ring-blue-200/40 font-semibold"
                        >Start</span
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
        width: 440px;
        height: 160px;
        border-radius: 0 0 28px 28px;
        transition:
            width 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0s,
            height 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s,
            border-radius 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s;
    }

    .compact-view {
        transform: scale(1);
        transition:
            opacity 0.3s ease 0s,
            transform 0.3s ease 0s;
    }

    .compact-view.is-hidden {
        opacity: 0;
        transform: scale(0.5);
        pointer-events: none;
    }

    .expanded-view {
        scroll-snap-type: x mandatory;
        -webkit-overflow-scrolling: touch;
        scrollbar-width: none;
        opacity: 0;
        transform: scale(0.9);
        pointer-events: none;
        transition:
            opacity 0.3s ease 0s,
            transform 0.3s ease 0s;
    }

    .expanded-view::-webkit-scrollbar {
        display: none;
    }

    .expanded-view.visible {
        opacity: 1;
        transform: scale(1);
        pointer-events: auto;
        transition:
            opacity 0.25s ease 0.35s,
            transform 0.25s ease 0.35s;
    }

    .expand-child {
        @apply flex flex-col w-full flex-[1_0_auto];
        padding: 12px 0;
        scroll-snap-align: start;
    }
</style>
