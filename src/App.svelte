<script lang="ts">
    import "./app.css";
    import "./macos.css";
    import { magnetic } from "./lib/magnetic";
    import { platform } from "./lib/platform";
    import { remaining } from "./lib/todos";
    import Todo from "./lib/Todo.svelte";

    import { invoke } from "@tauri-apps/api/core";

    import { onMount } from "svelte";

    const SIZE = {
        compact: { width: 480, height: 32 },
        open: { width: 480, height: 205 },
    };
    const TABS = ["Clock", "Todo", "Pomodoro", "Shelf"];

    let open = false;
    let wide = false;
    let windowTall = false;
    let activeTab = 0;
    let time = new Date();

    const COOLDOWN_MS = 400;
    let locked = false;

    let clockTimer: ReturnType<typeof setTimeout> | undefined;

    function scheduleClock() {
        time = new Date();
        clockTimer = setTimeout(
            scheduleClock,
            60_000 - (time.getTime() % 60_000) + 50,
        );
    }

    onMount(() => {
        invoke("low_memory").catch((error) =>
            console.error("low_memory failed:", error),
        );
        scheduleClock();

        const handleContextMenu = (e: MouseEvent) => e.preventDefault();

        const onVisibility = () => {
            clearTimeout(clockTimer);
            if (!document.hidden) scheduleClock();
        };

        const isTauri =
            window.location.protocol === "tauri:" ||
            window.location.hostname === "tauri.localhost";
        if (isTauri) {
            window.addEventListener("contextmenu", handleContextMenu);
        }
        document.addEventListener("visibilitychange", onVisibility);
        return () => {
            clearTimeout(clockTimer);
            document.removeEventListener("visibilitychange", onVisibility);
            window.removeEventListener("contextmenu", handleContextMenu);
        };
    });

    $: timeString = time.toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
    });
    $: dayString = time.toLocaleDateString([], {
        weekday: "short",
    });
    $: dateString = time.toLocaleDateString([], {
        month: "short",
        day: "numeric",
    });

    const nextFrame = () => new Promise(requestAnimationFrame);

    let hoverTimer: ReturnType<typeof setTimeout> | undefined;

    function onEnter() {
        clearTimeout(hoverTimer);
        hoverTimer = setTimeout(() => (wide = true), 100);
    }

    function onLeave() {
        clearTimeout(hoverTimer);
        wide = false;
    }

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
            windowTall = true;
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
        if (e.propertyName !== "width" || open || !windowTall) return;
        windowTall = false;
        invoke("resize_island", SIZE.compact);
    }
</script>

<main class="flex justify-center">
    <div
        role="button"
        tabindex="0"
        class="island relative text-white cursor-pointer"
        class:wide
        class:open
        on:click={toggle}
        on:keydown={buttonLike}
        on:mouseenter={onEnter}
        on:mouseleave={onLeave}
        on:transitionend={onTransitionEnd}
    >
        <div
            class="relative w-full h-full flex flex-col items-start font-sans overflow-hidden"
        >
            <!-- Compact view -->
            <div
                class="compact-view absolute top-2 z-20 w-full px-5 flex {platform ===
                'macos'
                    ? 'justify-start'
                    : 'justify-center'} items-center text-[13px] font-semibold tabular-nums"
                class:is-hidden={open}
            >
                {#if platform === "macos"}
                    <!-- render nothing -->
                {:else}
                    <span class="compact-wide"><span class="text-accent font-bold">{dayString}</span>, {dateString}</span>
                {/if}
                <span>{timeString}</span>
                <span class="compact-wide right">
                    {#if $remaining === 0}
                        all clear
                    {:else}
                        <span>
                            <span class="text-accent font-bold text-[13px]"
                                >{$remaining}</span
                            >
                            {$remaining === 1 ? "task" : "tasks"} left
                        </span>
                    {/if}
                </span>
            </div>

            <!-- Tab nav -->
            <div
                class="reveal top-nav w-full flex {platform === 'macos'
                    ? 'justify-between'
                    : 'justify-center'} items-center"
                class:shown={open}
            >
                {#if platform === "macos"}
                    <div class="flex items-center gap-1">
                        {#each TABS.slice(0, 2) as tab, index}
                            <button
                                type="button"
                                class="nav-pill"
                                class:active={activeTab === index}
                                on:click|stopPropagation={() =>
                                    (activeTab = index)}
                            >
                                {tab}
                            </button>
                        {/each}
                    </div>

                    <div class="flex items-center gap-1">
                        {#each TABS.slice(2) as tab, i}
                            {@const index = i + 2}
                            <button
                                type="button"
                                class="nav-pill"
                                class:active={activeTab === index}
                                on:click|stopPropagation={() =>
                                    (activeTab = index)}
                            >
                                {tab}
                            </button>
                        {/each}
                    </div>
                {:else}
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
                {/if}
            </div>

            <!-- Sliding panels -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="reveal panel-viewport relative w-full flex-1 overflow-hidden"
                class:shown={open}
                on:click|stopPropagation={() => {}}
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
                            <Todo />
                        </div>
                    </div>

                    <div class="panel">
                        <div class="panel-box">
                            <span class="text-[24px]">Pomodoro (unfinished)</span>
                            <span class="start-pill">Start</span>
                        </div>
                    </div>

                    <div class="panel">
                        <div class="panel-box justify-start!">
                            <span class="text-[13px] font-regular mt-0.5">Temporarily saved files (unfinished)</span>
                            <div class="border-2 border-white/35 border-dashed w-full h-full my-2 rounded-2xl"></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</main>
