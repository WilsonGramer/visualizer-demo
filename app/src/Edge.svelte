<script>
    import {
        BaseEdge,
        BezierEdge,
        EdgeLabel,
        getBezierPath,
        MarkerType,
        Position,
    } from "@xyflow/svelte";

    const {
        id,
        sourceX,
        sourceY,
        sourcePosition,
        targetX,
        targetY,
        targetPosition,
        markerEnd,
        style,
        label,
    } = $props();

    const [edgePath, labelX, labelY] = $derived(
        getBezierPath({
            sourceX,
            sourceY,
            sourcePosition,
            targetX,
            targetY,
            targetPosition,
        }),
    );
</script>

<BaseEdge path={edgePath} {markerEnd} style="stroke-width: 2px; stroke-dasharray: 2px;" />

<EdgeLabel x={labelX} y={labelY} style="background: #fff8; font-size: 85%;">
    {label}
</EdgeLabel>
