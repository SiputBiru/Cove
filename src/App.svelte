<script lang="ts">
    import "./app.css";
    import "./macos.css";
    import { platform } from "./lib/platform";
    import { remaining } from "./lib/stores/todos";

    import ClockPanel from "./lib/panels/Clock.svelte";
    import TodoPanel from "./lib/panels/Todo.svelte";
    import ShelfPanel from "./lib/panels/Shelf.svelte";
    import SettingsPanel from "./lib/panels/Settings.svelte";

    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { getCurrentWindow } from "@tauri-apps/api/window";

    import { onMount } from "svelte";
    import {
        Clock,
        CheckSquare,
        Timer,
        FolderArchive,
        Settings,
    } from "lucide-svelte";

    const SIZE = {
        compact: { width: 480, height: 32 },
        open: { width: 480, height: 215 },
    };

    const TABS = [
        { label: "Clock", icon: Clock },
        { label: "Todo", icon: CheckSquare },
        { label: "Pomodoro", icon: Timer, unfinished: true },
        { label: "Shelf", icon: FolderArchive, unfinished: true },
        { label: "Settings", icon: Settings },
    ];

    const SETTINGS_INDEX = TABS.length - 1;
    
    let open = false;
    let wide = false;
    let windowTall = false;
    let activeTab = 0;
    let time = new Date();

    let launchAtLogin = true;
    let use24HourClock = true;
    let alwaysTiny = false;

    let fs = false;
    let peek = false;

    $: tiny = (fs || alwaysTiny) && !open;
    $: stowed = fs && !open && !peek;
    $: if (fs) wide = false;
    $: if (!fs && !alwaysTiny) peek = false;

    let wasTiny = false;

    const COOLDOWN_STANDARD = 400;
    const COOLDOWN_TINY = 550;
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
        scheduleClock();

        const unlistenFs = listen<boolean>(
            "fullscreen",
            (e) => (fs = e.payload),
        );
        const unlistenPeek = listen<boolean>("peek", (e) => (peek = e.payload));
        invoke<boolean>("fullscreen_mode")
            .then((value) => (fs = value))
            .catch(() => {});

        const unlistenOutside = listen("outside_click", () => {
            if (open && !locked) toggle();
        });

        const appWindow = getCurrentWindow();
        const unlistenFocus = appWindow.onFocusChanged(
            ({ payload: focused }) => {
                if (!focused && !locked) {
                    if (open) {
                        toggle();
                    } else if (peek) {
                        peek = false;
                    }
                }
            },
        );

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
        hour12: !use24HourClock,
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
        if (tiny || alwaysTiny) return;
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

        activeTab = Math.max(
            0,
            Math.min(TABS.length - 1, activeTab + Math.sign(delta)),
        );
    }

    async function toggle() {
        if (locked) return;
        locked = true;

        const startingFromTiny = tiny;
        if (startingFromTiny) {
            wasTiny = true;
        }

        const currentCooldown = startingFromTiny
            ? COOLDOWN_TINY
            : COOLDOWN_STANDARD;

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
            setTimeout(() => {
                locked = false;
                wasTiny = false;
            }, currentCooldown);
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
        class:was-tiny={wasTiny}
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
                    <span class="compact-wide">
                        <span class="text-accent font-bold">{dayString}</span>, {dateString}
                    </span>
                {/if}
                <div class="relative flex items-center gap-1.5">
                    <span>{timeString}</span>
                </div>
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
                class="reveal top-nav w-full flex justify-between items-center px-3"
                class:shown={open}
                on:wheel|preventDefault={scrollNav}
            >
                <!-- Left Nav Group -->
                <div class="flex items-center gap-1.5">
                    {#each TABS.slice(0, SETTINGS_INDEX) as tab, index}
                        <button
                            type="button"
                            tabindex="-1"
                            class="nav-pill group relative flex items-center justify-center"
                            class:active={activeTab === index}
                            on:click|stopPropagation={() => (activeTab = index)}
                        >
                            <svelte:component
                                this={tab.icon}
                                class="icon {tab.unfinished ? 'opacity-60' : ''} {activeTab === index
                                    ? 'text-accent scale-110'
                                    : 'text-white/70 group-hover:text-white group-hover:scale-110'}"
                            />
                        </button>
                    {/each}
                </div>

                <!-- Right Nav Group (Settings) -->
                {#if TABS[SETTINGS_INDEX]}
                    {@const settingsIndex = SETTINGS_INDEX}
                    <button
                        type="button"
                        tabindex="-1"
                        class="nav-pill group relative flex items-center justify-center"
                        class:active={activeTab === settingsIndex}
                        on:click|stopPropagation={() =>
                            (activeTab = settingsIndex)}
                        title={TABS[settingsIndex].label}
                    >
                        <svelte:component
                            this={TABS[settingsIndex].icon}
                            class="icon {activeTab === settingsIndex
                                ? 'text-accent scale-110'
                                : 'text-white/70 group-hover:text-white group-hover:scale-110'}"
                        />
                    </button>
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
                    <ClockPanel {timeString} {dateString} />

                    <div class="panel">
                        <div class="panel-box">
                            <TodoPanel />
                        </div>
                    </div>

                    <div class="panel">
                        <div class="panel-box gap-1.5">
                            <Timer class="size-6 text-accent/80" />
                            <span class="text-sm font-semibold">Pomodoro</span>
                            <span
                                class="text-[11px] text-[#8e8e93]"
                                >Unfinished · coming in v0.7</span
                            >
                        </div>
                    </div>

                    <ShelfPanel/>

                    <SettingsPanel
                        bind:use24HourClock
                        bind:alwaysTiny
                        bind:launchAtLogin
                    />
                </div>
            </div>
        </div>
    </div>
</main>
