<script>
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
    import Node from "./Node.svelte";
    import Edge from "./Edge.svelte";
    import * as dagre from "@dagrejs/dagre";

    const props = $props();

    const sharedProps = {
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

    const { fitView } = useSvelteFlow();

    const layouted = $derived.by(() => {
        const { nodes, edges, clusters } = $state.snapshot(props);

        const nodeSeparation = 150;
        const nodeLabelFontSize = 10;
        const nodePadding = 30;

        try {
            const g = new dagre.graphlib.Graph({ compound: true });
            g.setGraph({ nodesep: 150 });
            g.setDefaultEdgeLabel(() => ({}));

            for (const node of nodes) {
                g.setNode(node.id, {
                    width: node.data.source.length * nodeLabelFontSize + nodePadding,
                    height: nodePadding * 2,
                });
            }

            const edgeData = {};
            for (const edge of edges) {
                g.setEdge(edge.from, edge.to, {});
                edgeData[`edge-${edge.from}-${edge.to}`] = edge;
            }

            for (const cluster of clusters) {
                g.setNode(cluster.id, {});

                for (const node of cluster.nodes) {
                    g.setParent(node, cluster.id);
                }
            }

            dagre.layout(g);

            requestAnimationFrame(() => {
                fitView();
            });

            return {
                nodes: g.nodes().flatMap((id) => {
                    const cluster = g.node(id);
                    const children = g.children(id);

                    if (children.length === 0) {
                        return [];
                    }

                    return [
                        {
                            ...sharedProps,
                            id,
                            type: "custom-group",
                            data: clusters.find((n) => n.id === id),
                            position: {
                                x: cluster.x - cluster.width / 2,
                                y: cluster.y - cluster.height / 2,
                            },
                            width: cluster.width,
                            height: cluster.height,
                        },
                        ...children.map((id) => {
                            const node = g.node(id);

                            return {
                                ...sharedProps,
                                id,
                                type: "custom-node",
                                data: nodes.find((n) => n.id === id).data,
                                position: {
                                    x: node.x - node.width / 2,
                                    y: node.y - node.height / 2,
                                },
                                width: node.width,
                                height: node.height,
                            };
                        }),
                    ];
                }),
                edges: g.edges().map((edge) => {
                    const id = `edge-${edge.v}-${edge.w}`;

                    return {
                        ...sharedProps,
                        id,
                        type: "custom-edge",
                        source: edge.v,
                        target: edge.w,
                        label: edgeData[id].label,
                        zIndex: 1,
                        markerEnd: { type: MarkerType.ArrowClosed },
                    };
                }),
            };
        } catch (e) {
            console.error(e);
            return { nodes: [], edges: [] };
        }
    });
</script>

<div class="h-[600px] shrink-0">
    <SvelteFlow {nodeTypes} {edgeTypes} nodes={layouted.nodes} edges={layouted.edges} fitView>
        <Controls />
        <Background />
    </SvelteFlow>
</div>
