<script lang="ts">
    import "./app.css";
    import "./macos.css";
    import { magnetic } from "./lib/magnetic";
    import { platform } from "./lib/platform";
    import { remaining } from "./lib/todos";
    import Todo from "./lib/Todo.svelte";

    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { getCurrentWindow } from "@tauri-apps/api/window";

    import { onMount } from "svelte";
    import { Clock, CheckSquare, Timer, FolderArchive } from "lucide-svelte";

    const SIZE = {
        compact: { width: 480, height: 32 },
        open: { width: 480, height: 205 },
    };

    const TABS = [
        { label: "Clock", icon: Clock },
        { label: "Todo", icon: CheckSquare },
        { label: "Pomodoro", icon: Timer },
        { label: "Shelf", icon: FolderArchive },
    ];

    let open = false;
    let wide = false;
    let windowTall = false;
    let activeTab = 0;
    let time = new Date();

    let fs = false;

    let peek = false;
    $: tiny = fs && !open;
    $: stowed = fs && !open && !peek;
    $: if (fs) wide = false;
    $: if (!fs) peek = false;

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

        const unlistenFs = listen<boolean>("fullscreen", (e) => (fs = e.payload));
        const unlistenPeek = listen<boolean>("peek", (e) => (peek = e.payload));
        invoke<boolean>("fullscreen_mode")
            .then((value) => (fs = value))
            .catch(() => {});

        const unlistenOutside = listen("outside_click", () => {
            if (open && !locked) toggle();
        });

        const appWindow = getCurrentWindow();
        const unlistenFocus = appWindow.onFocusChanged(({ payload: focused }) => {
            if (!focused && !locked) {
                if (open) {
                    toggle();
                } else if (peek) {
                    peek = false;
                }
            }
        });

        const handleContextMenu = (e: MouseEvent) => e.preventDefault();

        const blockKeys = (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "p") {
                e.preventDefault();
                e.stopPropagation();
                return;
            }
            if (e.key !== "Tab") return;
            const el = e.target as HTMLElement;
            if (el.tagName === "INPUT" || el.tagName === "TEXTAREA") return;
            e.preventDefault();
        };

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
        window.addEventListener("keydown", blockKeys, true);
        document.addEventListener("visibilitychange", onVisibility);
        return () => {
            clearTimeout(clockTimer);
            unlistenFs.then((stop) => stop());
            unlistenPeek.then((stop) => stop());
            unlistenOutside.then((stop) => stop());
            unlistenFocus.then((stop) => stop());
            document.removeEventListener("visibilitychange", onVisibility);
            window.removeEventListener("contextmenu", handleContextMenu);
            window.removeEventListener("keydown", blockKeys, true);
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
        if (tiny) return;
        clearTimeout(hoverTimer);
        hoverTimer = setTimeout(() => (wide = true), 150);
    }

    function onLeave() {
        clearTimeout(hoverTimer);
        hoverTimer = setTimeout(() => (wide = false), 350);
    }

    let wheelLocked = false;

    function scrollNav(event: WheelEvent) {
        if (wheelLocked) return;

        const delta = event.deltaY || event.deltaX;
        if (Math.abs(delta) < 10) return;

        wheelLocked = true;
        setTimeout(() => (wheelLocked = false), 350);

        activeTab = Math.max(0, Math.min(TABS.length - 1, activeTab + Math.sign(delta)));
    }

    async function toggle() {
        if (locked) return;
        locked = true;

        try {
            if (open) {
                open = false;
                return;
            }
            await invoke("set_open", { open: true });
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
        invoke("set_open", { open: false });
    }
</script>

<main class="flex justify-center">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
        role="button"
        tabindex="0"
        class="island relative text-white cursor-pointer"
        class:wide
        class:open
        class:tiny
        class:stowed
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
                class:is-hidden={open || tiny}
            >
                {#if platform === "macos"}
                    <!-- render nothing -->
                {:else}
                    <span class="compact-wide"
                        ><span class="text-accent font-bold">{dayString}</span>, {dateString}</span
                    >
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
                on:wheel|preventDefault={scrollNav}
            >
                {#if platform === "macos"}
                    <div class="flex items-center gap-1">
                        {#each TABS.slice(0, 2) as tab, index}
                            <button
                                type="button"
                                tabindex="-1"
                                class="nav-pill group relative flex items-center justify-center"
                                class:active={activeTab === index}
                                on:click|stopPropagation={() =>
                                    (activeTab = index)}
                                title={tab.label}
                            >
                                <svelte:component
                                    this={tab.icon}
                                    class="icon transition-all duration-300 ease-out {activeTab === index ? 'text-accent scale-110' : 'text-white/70 group-hover:text-white group-hover:scale-110'}"
                                />
                            </button>
                        {/each}
                    </div>

                    <div class="flex items-center gap-1">
                        {#each TABS.slice(2) as tab, i}
                            {@const index = i + 2}
                            <button
                                type="button"
                                tabindex="-1"
                                class="nav-pill group relative flex items-center justify-center"
                                class:active={activeTab === index}
                                on:click|stopPropagation={() =>
                                    (activeTab = index)}
                                title={tab.label}
                            >
                                <svelte:component
                                    this={tab.icon}
                                    class="icon transition-all duration-300 ease-out {activeTab === index ? 'text-accent scale-110' : 'text-white/70 group-hover:text-white group-hover:scale-110'}"
                                />
                            </button>
                        {/each}
                    </div>
                {:else}
                    {#each TABS as tab, index}
                        <button
                            type="button"
                            tabindex="-1"
                            class="nav-pill group relative flex items-center justify-center"
                            class:active={activeTab === index}
                            on:click|stopPropagation={() => (activeTab = index)}
                            title={tab.label}
                        >
                            <svelte:component
                                this={tab.icon}
                                class="icon transition-all duration-300 ease-out {activeTab === index ? 'text-accent scale-110' : 'text-white/70 group-hover:text-white group-hover:scale-110'}"
                            />
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
                            <span class="text-[24px]"
                                >Pomodoro (unfinished)</span
                            >
                            <span class="start-pill">Start</span>
                        </div>
                    </div>

                    <div class="panel">
                        <div class="panel-box justify-start!">
                            <div
                                class="flex justify-center items-center border-2 border-white/35 border-dashed w-full h-full m-2 rounded-2xl"
                            >
                                <span
                                    class="text-[13px] font-light text-white/50 mt-0.5"
                                    >Temporarily saved files (unfinished)</span
                                >
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</main>
