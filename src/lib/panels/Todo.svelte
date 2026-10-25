<script lang="ts">
    import { flip } from "svelte/animate";
    import { slide } from "svelte/transition";
    import {
        todos,
        remaining,
        addTask,
        toggleTask,
        removeTask,
        clearDone,
    } from "../stores/todos";

    let draft = "";

    $: sorted = [...$todos].sort((a, b) => Number(a.done) - Number(b.done));
    $: hasDone = $remaining < $todos.length;

    function add() {
        if (addTask(draft)) draft = "";
    }

    function onKeydown(e: KeyboardEvent) {
        if (e.key === "Enter" && !e.isComposing) {
            e.preventDefault();
            add();
        } else if (e.key === "Escape") {
            (e.currentTarget as HTMLInputElement).blur();
        }
    }
</script>

<div class="flex h-full min-h-0 w-full flex-col gap-1.5">
    <!-- Add row -->
    <div class="flex shrink-0 items-center gap-2">
        <input
            type="text"
            bind:value={draft}
            on:keydown={onKeydown}
            placeholder="Type a task, press enter to add..."
            maxlength="120"
            aria-label="Add a task"
            class="select-text min-w-0 flex-1 rounded-full bg-black/30 px-3 py-1 text-[13px] text-white outline-none transition-shadow placeholder:text-white/40 focus:ring-[1.5px] focus:ring-accent/60"
        />
        <span
            class="shrink-0 text-[11px] font-semibold tracking-wide tabular-nums text-white/65"
        >
            {$remaining} Left
        </span>
        {#if hasDone}
            <button
                type="button"
                class="shrink-0 cursor-pointer text-[11px] font-semibold uppercase tracking-wide text-accent/80 transition-colors hover:text-accent"
                on:click={clearDone}
            >
                Clear
            </button>
        {/if}
    </div>

    <!-- List -->
    <ul
        class="min-h-0 flex-1 overflow-y-auto scrollbar-none [&::-webkit-scrollbar]:hidden"
    >
        {#each sorted as task (task.id)}
            <li
                class="group flex items-center gap-2 rounded-lg pr-1 pl-2.5 py-1 transition-colors hover:bg-white/5"
                animate:flip={{ duration: 200 }}
                transition:slide={{ duration: 180 }}
            >
                <button
                    type="button"
                    role="checkbox"
                    aria-checked={task.done}
                    aria-label={task.done ? "Mark as not done" : "Mark as done"}
                    class="flex size-4 shrink-0 cursor-pointer items-center justify-center rounded-full border transition-all duration-150 active:scale-90 {task.done
                        ? 'border-accent bg-accent'
                        : 'border-white/40 hover:border-white/70'}"
                    on:click={() => toggleTask(task.id)}
                >
                    {#if task.done}
                        <svg viewBox="0 0 12 12" class="size-2.5 text-black">
                            <path
                                d="M2.5 6.5l2.2 2.2 4.8-5"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    {/if}
                </button>

                <span
                    class="min-w-0 flex-1 truncate text-[13px] transition-colors {task.done
                        ? 'text-white/40 line-through'
                        : 'text-white'}"
                    title={task.text}
                >
                    {task.text}
                </span>

                <button
                    type="button"
                    aria-label="Delete task"
                    class="shrink-0 cursor-pointer rounded-full px-1.5 text-[18px] leading-none text-white/40 opacity-0 transition-opacity hover:text-white focus-visible:opacity-100 group-hover:opacity-100"
                    on:click={() => removeTask(task.id)}
                >
                    &times;
                </button>
            </li>
        {:else}
            <li
                class="py-4 text-center text-[12px] uppercase tracking-[0.8px] text-white/30"
            >
                All clear
            </li>
        {/each}
    </ul>
</div>
