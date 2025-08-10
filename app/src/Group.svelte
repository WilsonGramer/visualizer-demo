<script module>
    export const paddingTop = 30;
</script>

<script>
    const { data, width, height } = $props();
</script>

<div
    style:width="{width}px"
    style:height="{height}px"
    class={[
        "relative border rounded-lg shadow-black/5 overflow-clip max-w-full max-h-full flex text-center",
        data.labels.length > 1
            ? "border-red-500 bg-red-500/10 shadow-red-500/20 shadow-md"
            : " border-blue-500 bg-blue-500/10",
    ]}
>
    <code class="text-center w-full text-blue-500 font-medium text-[85%]">
        {#each data.labels as label, index}
            {#if index > 0}
                <span class="text-red-500">&nbsp;or&nbsp;</span>
            {/if}{label}
        {/each}
    </code>
</div>
