<script>
    import { Handle, Position } from "@xyflow/svelte";

    const { data, width, height } = $props();
</script>

<div
    style:width="{width}px"
    style:height="{height}px"
    class="relative border border-gray-300 rounded-lg shadow-md shadow-black/5 overflow-clip max-w-full max-h-full bg-white flex text-center items-center"
>
    <code class="text-center w-full">{data.source}</code>

    {#if data.comments}
        <p>{data.comments}</p>
    {/if}
</div>

<Handle type="target" position={Position.Top} class="opacity-0" />
<Handle type="source" position={Position.Bottom} class="opacity-0" />
