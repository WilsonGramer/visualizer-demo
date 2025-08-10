<script>
    import {
        BaseEdge,
        BezierEdge,
        EdgeLabel,
        getBezierPath,
        MarkerType,
        Position,
    } from "@xyflow/svelte";
    import * as changeCase from "change-case";

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

    const formattedLabel = $derived(changeCase.noCase(label).split(" in ")[0]);
</script>

<BaseEdge path={edgePath} {markerEnd} style="stroke-width: 2px; stroke-dasharray: 2px;" />

<EdgeLabel
    x={labelX}
    y={labelY}
    style="background: #fff8; font-size: 85%; border-radius: 12px; padding: 2px 8px;"
>
    {formattedLabel}
</EdgeLabel>
