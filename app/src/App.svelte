<script>
    import initWasm, { run } from "app-wasm";
    import { AnsiUp } from "ansi_up";
    import { SvelteFlowProvider } from "@xyflow/svelte";
    import Graph from "./Graph.svelte";
    import Editor from "./Editor.svelte";
    import { onMount } from "svelte";

    const ansi = new AnsiUp();

    const debounce = (timeout, f) => {
        let timeoutId;
        return () => {
            window.clearTimeout(timeoutId);
            timeoutId = window.setTimeout(f, timeout);
        };
    };

    const metaKey = navigator.platform.startsWith("Mac") ? "⌘" : "Ctrl";

    let status;
    let graph;
    let output;

    let code = $state("");
    let selections = $state([]);
    let graphData = $state();

    const update = debounce(300, async () => {
        const url = new URL(window.location.href);
        url.searchParams.set("code", code);
        window.history.replaceState({}, "", url.toString());

        await initWasm();

        const filter = Uint32Array.from(selections.filter(([from, to]) => from !== to).flat());

        status.innerText =
            filter.length > 0
                ? `Filtering by selection (hold ${metaKey} to select multiple)`
                : "Showing all code (select code to filter)";

        const [outputString, outputGraphData] = run(code, filter);
        output.innerHTML = ansi.ansi_to_html(outputString);
        graphData = outputGraphData;
    });

    $effect(() => {
        const query = new URLSearchParams(window.location.search);
        if (query.has("code")) {
            code = query.get("code");
        }
    });

    $effect(() => {
        code;
        selections;
        update();
    });
</script>

<div class="flex flex-col w-screen h-screen p-[10px] gap-[10px]">
    <div class="relative">
        <a href="https://www.wipple.org" class="flex flex-row items-center gap-[10px]">
            <img src="logo.svg" class="size-[32px]" alt="" />
            <p class="font-semibold">Wipple</p>
        </a>

        <div class="absolute inset-0 flex items-center justify-center">
            <p class="text-center font-semibold text-lg">Visualizer Demo</p>
        </div>
    </div>

    <div class="relative flex-1 flex flex-row gap-[10px] min-h-0">
        <div class="flex flex-col flex-1 gap-[10px] max-w-[500px]">
            <div
                class="flex-2 border-[1.5px] border-black/5 rounded-lg p-[14px] font-mono resize-none focus:outline-blue-500"
            >
                <Editor bind:code bind:selections />
            </div>

            <pre
                bind:this={output}
                class="flex-1 border-[1.5px] border-black/5 rounded-lg overflow-scroll p-[10px]"></pre>
        </div>

        <div
            class="flex-2 border-[1.5px] border-black/5 rounded-lg flex flex-col gap-[20px] p-[10px]"
        >
            <p bind:this={status} class="text-sm text-black/50"></p>

            {#if graphData != null}
                <SvelteFlowProvider>
                    <Graph {...graphData} />
                </SvelteFlowProvider>
            {/if}
        </div>
    </div>

    <div class="flex items-center justify-center">
        <p class="text-sm text-current/70">
            Made by
            <a href="https://gramer.dev" class="font-semibold">Wilson Gramer</a>
        </p>
    </div>
</div>
