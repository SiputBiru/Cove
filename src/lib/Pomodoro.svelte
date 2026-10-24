<script lang="ts">
    import { createEventDispatcher, onDestroy } from "svelte";
    import { RotateCcw, SkipForward } from "lucide-svelte";
    import PixelIcon from "./PixelIcon.svelte";

    export let focusMinutes = 25;
    export let breakMinutes = 5;
    export let longBreakMinutes = 15;
    export let sessionsBeforeLongBreak = 4;
    // start the next phase automatically when one finishes
    export let autoStart = false;

    export let running = false;
    export let mode: "focus" | "break" = "focus";
    export let remaining = focusMinutes * 60; // seconds

    const dispatch = createEventDispatcher<{
        complete: { mode: "focus" | "break"; sessions: number };
    }>();

    let completed = 0; // finished focus sessions
    let total = remaining; // length of the current phase, in seconds
    let endAt = 0; // timestamp the phase ends at (keeps time correct even if the webview is suspended)
    let timer: ReturnType<typeof setInterval> | undefined;

    function phaseLength(m: "focus" | "break"): number {
        if (m === "focus") return focusMinutes * 60;
        const long = completed > 0 && completed % sessionsBeforeLongBreak === 0;
        return (long ? longBreakMinutes : breakMinutes) * 60;
    }

    function setMode(m: "focus" | "break") {
        mode = m;
        total = phaseLength(m);
        remaining = total;
    }

    function tick() {
        remaining = Math.max(0, Math.ceil((endAt - Date.now()) / 1000));
        if (remaining === 0) finish();
    }

    function start() {
        endAt = Date.now() + remaining * 1000;
        running = true;
        clearInterval(timer);
        timer = setInterval(tick, 250);
    }

    function pause() {
        clearInterval(timer);
        running = false;
    }

    function toggle() {
        running ? pause() : start();
    }

    function reset() {
        pause();
        remaining = total;
    }

    function next() {
        setMode(mode === "focus" ? "break" : "focus");
    }

    function finish() {
        pause();
        if (mode === "focus") completed++;
        dispatch("complete", { mode, sessions: completed });
        next();
        if (autoStart) start();
    }

    // skip the current phase without counting it as a finished session
    function skip() {
        pause();
        next();
    }

    onDestroy(() => clearInterval(timer));

    $: progress = total ? 1 - remaining / total : 0;
    $: mm = String(Math.floor(remaining / 60)).padStart(2, "0");
    $: ss = String(remaining % 60).padStart(2, "0");
    // how many dots of the current cycle are filled
    $: cycle =
        completed > 0 && completed % sessionsBeforeLongBreak === 0 && mode === "break"
            ? sessionsBeforeLongBreak
            : completed % sessionsBeforeLongBreak;
</script>

<div class="flex flex-col items-center w-full gap-1">
    <!-- mode label + cycle dots -->
    <div class="flex items-center gap-2 h-3.5">
        {#if running}
            <PixelIcon variant={mode === "focus" ? "thinking" : "responding"} />
        {/if}
        <span class="text-[11px] uppercase tracking-[0.8px] text-[#8e8e93]">
            {mode === "focus" ? "Focus" : "Break"}
        </span>
        <span class="flex gap-1">
            {#each Array(sessionsBeforeLongBreak) as _, i}
                <span
                    class="size-1 rounded-full transition-colors duration-300 {i < cycle
                        ? 'bg-accent'
                        : 'bg-white/20'}"
                ></span>
            {/each}
        </span>
    </div>

    <!-- time -->
    <span class="title">{mm}:{ss}</span>

    <!-- progress -->
    <div class="w-36 h-[3px] rounded-full bg-white/10 overflow-hidden">
        <div
            class="h-full bg-accent rounded-full transition-[width] duration-300 ease-linear"
            style="width: {progress * 100}%"
        ></div>
    </div>

    <!-- controls -->
    <div class="flex items-center gap-3 mt-1.5">
        <button
            type="button"
            tabindex="-1"
            title="Reset"
            class="p-1.5 rounded-full text-white/50 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
            on:click={reset}
        >
            <RotateCcw class="size-3.5" />
        </button>

        <button type="button" tabindex="-1" class="start-pill cursor-pointer" on:click={toggle}>
            {running ? "Pause" : "Start"}
        </button>

        <button
            type="button"
            tabindex="-1"
            title="Skip"
            class="p-1.5 rounded-full text-white/50 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
            on:click={skip}
        >
            <SkipForward class="size-3.5" />
        </button>
    </div>
</div>
