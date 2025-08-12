<script>
    import { EditorState } from "@codemirror/state";
    import { EditorView, placeholder } from "@codemirror/view";
    import { basicSetup } from "codemirror";
    import { onMount } from "svelte";

    let { code = $bindable(), selections = $bindable() } = $props();

    let editor;

    let view;
    onMount(() => {
        view = new EditorView({
            doc: code,
            parent: editor,
            extensions: [
                basicSetup,
                placeholder("Write your code here..."),
                EditorView.lineWrapping,
                EditorState.allowMultipleSelections.of(true),
                EditorView.updateListener.of((update) => {
                    if (update.docChanged) {
                        code = update.state.doc.toString();
                    }

                    if (update.selectionSet) {
                        selections = update.state.selection.ranges.map((range) => [
                            range.from,
                            range.to,
                        ]);
                    }
                }),
            ],
        });
    });

    $effect(() => {
        if (code === view.state.doc.toString()) {
            return;
        }

        view.dispatch({
            changes: {
                from: 0,
                to: view.state.doc.length,
                insert: code,
            },
        });
    });
</script>

<div bind:this={editor} class="flex-1"></div>

<style>
    :global(.cm-editor) {
        width: 100%;
        height: 100%;
    }
</style>
