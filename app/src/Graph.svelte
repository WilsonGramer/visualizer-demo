<script>
    import ELK from "elkjs/lib/elk.bundled.js";
    import {
        Background,
        Controls,
        MarkerType,
        MiniMap,
        Position,
        SvelteFlow,
        useSvelteFlow,
    } from "@xyflow/svelte";
    import Group, { paddingTop } from "./Group.svelte";
    import Node, { width, height } from "./Node.svelte";
    import Edge from "./Edge.svelte";
    import * as changeCase from "change-case";

    const props = $props();

    const sharedProps = {
        position: { x: 0, y: 0 },
        selectable: false,
        draggable: false,
    };

    const nodeTypes = {
        "custom-group": Group,
        "custom-node": Node,
    };

    const edgeTypes = {
        "custom-edge": Edge,
    };

    // Adapted from https://github.com/mermaid-js/mermaid/blob/9322771b5ce48ab45b9ba1fc8b5651e37bb673cb/packages/mermaid-layout-elk/src/render.ts#L765
    const defaultLayoutOptions = {
        "elk.algorithm": "layered",
        "elk.direction": "RIGHT",
        "elk.hierarchyHandling": "INCLUDE_CHILDREN",
        "elk.layered.crossingMinimization.forceNodeModelOrder": true,
        "elk.layered.unnecessaryBendpoints": true,
        "elk.edgeRouting": "POLYLINE",
        "spacing.baseValue": 10,
    };

    const { fitView } = useSvelteFlow();

    const elk = new ELK({ defaultLayoutOptions });

    let layouted = $state({ nodes: [], edges: [] });

    const onLayout = async () => {
        const graph = {
            id: "root",
            children: props.clusters.map((cluster) => ({
                ...sharedProps,
                layoutOptions: {
                    "elk.padding": `top=${paddingTop},left=10,right=10,bottom=10`,
                },
                id: cluster.id,
                data: cluster,
                children: props.nodes
                    .filter((node) => cluster.nodes.includes(node.id))
                    .map((node) => ({
                        ...sharedProps,
                        id: node.id,
                        data: node.data,
                        width,
                        height,
                    })),
            })),
            edges: props.edges.map((edge) => {
                const label = changeCase.noCase(edge.label).split(" in ")[0];

                return {
                    ...sharedProps,
                    id: `edge-${edge.from}-${edge.to}`,
                    source: edge.from,
                    target: edge.to,
                    label,
                    labels: [{ text: label, width: 100 }],
                    zIndex: 1,
                    markerEnd: { type: MarkerType.ArrowClosed },
                };
            }),
        };

        try {
            const layoutedGraph = await elk.layout(graph);

            layouted.nodes = layoutedGraph.children.flatMap((cluster) => [
                {
                    ...cluster,
                    type: "custom-group",
                    position: { x: cluster.x, y: cluster.y },
                },
                ...cluster.children?.map((node) => ({
                    ...node,
                    type: "custom-node",
                    position: { x: node.x + cluster.x, y: node.y + cluster.y },
                })),
            ]);

            layouted.edges = layoutedGraph.edges.map((edge) => ({
                ...edge,
                type: "custom-edge",
                data: { sections: edge.sections, labels: edge.labels },
            }));
        } catch (e) {
            console.error(e);
        }

        fitView();
    };

    $effect(() => {
        onLayout();
    });
</script>

<div class="h-[600px] shrink-0">
    <SvelteFlow {nodeTypes} {edgeTypes} nodes={layouted.nodes} edges={layouted.edges} fitView>
        <Controls />
        <Background />
    </SvelteFlow>
</div>
