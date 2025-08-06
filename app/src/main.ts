import mermaid from "mermaid";
import initWasm, { run } from "app-wasm";
import "./style.css";
import { AnsiUp } from "ansi_up";

mermaid.initialize({ startOnLoad: false });
const ansi = new AnsiUp();

const debounce = (timeout: number, f: () => void) => {
    let timeoutId: number | undefined;
    return () => {
        window.clearTimeout(timeoutId);
        timeoutId = window.setTimeout(f, timeout);
    };
};

const code = document.querySelector("#code") as HTMLTextAreaElement;
const status = document.querySelector("#status") as HTMLParagraphElement;
const graph = document.querySelector("#graph") as HTMLDivElement;
const output = document.querySelector("#output") as HTMLPreElement;

const query = new URLSearchParams(window.location.search);
if (query.has("code")) {
    code.value = query.get("code")!;
}

const update = debounce(300, async () => {
    const url = new URL(window.location.href);
    url.searchParams.set("code", code.value);
    window.history.replaceState({}, "", url.toString());

    await initWasm();

    const filter =
        code.selectionStart !== code.selectionEnd
            ? Uint32Array.from([code.selectionStart, code.selectionEnd])
            : undefined;

    status.innerText = filter
        ? "Filtering by selection"
        : "Showing all code (select code to filter)";

    const [outputString, graphString] = run(code.value, filter);

    if (graphString) {
        const { svg } = await mermaid.render("graphSvg", graphString);
        graph.innerHTML = svg;
    } else {
        graph.innerHTML = "";
    }

    output.innerHTML = ansi.ansi_to_html(outputString);
});

code.addEventListener("input", update);
code.addEventListener("selectionchange", update);

update();
