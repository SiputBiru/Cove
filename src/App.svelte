<script lang="ts">
    import "./app.css";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { magnetic } from "./lib/magnetic";
    import "@fontsource-variable/inter/wght.css";

    const SIZE = {
        compact: { width: 480, height: 32 },
        open: { width: 480, height: 160 },
    };
    const TABS = ["Clock", "Todo", "Pomodoro"];

    let open = false;
    let activeTab = 0;
    let time = new Date();

    const COOLDOWN_MS = 750;
    let locked = false;

    onMount(() => {
        const timer = setInterval(() => (time = new Date()), 1000);
        return () => clearInterval(timer);
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
        if (locked) return;
        locked = true;

        try {
            if (open) {
                open = false;
                return;
            }
            await invoke("resize_island", SIZE.open);
            await nextFrame();
            await nextFrame();
            open = true;
        } finally {
            setTimeout(() => (locked = false), COOLDOWN_MS);
        }
    }

    function buttonLike(e: KeyboardEvent) {
        if (e.target !== e.currentTarget) return;
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            toggle();
        }
    }

    function onTransitionEnd(e: TransitionEvent) {
        if (e.target !== e.currentTarget) return;
        if (e.propertyName === "width" && !open) {
            invoke("resize_island", SIZE.compact);
        }
    }
</script>

<main class="flex justify-center">
    <div
        role="button"
        tabindex="0"
        class="island relative bg-black text-white cursor-pointer will-change-[width,height,border-radius] transform-gpu"
        class:open
        on:click={toggle}
        on:keydown={buttonLike}
        on:transitionend={onTransitionEnd}
    >
        <div
            class="relative w-full h-full flex flex-col items-center font-sans overflow-hidden"
        >
            <!-- Compact view -->
            <div
                class="compact-view absolute top-2 z-20 w-full px-5 flex justify-between items-center text-[13px] font-semibold tabular-nums"
                class:is-hidden={open}
            >
                <div class="flex justify-center items-center ml-0.5">
                    <div
                        class="absolute size-2.5 bg-blue-100 rounded-full"
                    ></div>
                    <div
                        class="absolute size-3 bg-blue-100 rounded-full animate-ping"
                    ></div>
                </div>
                <span>{timeString}</span>
            </div>

            <!-- Tab nav -->
            <div class="top-nav reveal" class:shown={open}>
                {#each TABS as tab, index}
                    <button
                        type="button"
                        class="nav-pill"
                        class:active={activeTab === index}
                        on:click|stopPropagation={() => (activeTab = index)}
                    >
                        {tab}
                    </button>
                {/each}
            </div>

            <!-- Sliding panels -->
            <div
                class="reveal relative w-full flex-1 overflow-hidden"
                class:shown={open}
            >
                <div
                    class="slider-track"
                    style="transform: translateX(-{activeTab * 100}%);"
                >
                    <div class="panel">
                        <div class="panel-box">
                            <div
                                class="flex flex-col items-center"
                                use:magnetic
                            >
                                <span class="title">{timeString}</span>
                                <span class="subtitle">{dateString}</span>
                            </div>
                        </div>
                    </div>

                    <div class="panel">
                        <div class="panel-box">
                            <span class="title">Todo Tasks</span>
                            <span class="subtitle">Example</span>
                        </div>
                    </div>

                    <div class="panel">
                        <div class="panel-box">
                            <span class="title">Pomodoro</span>
                            <span class="start-pill">Start</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</main>
