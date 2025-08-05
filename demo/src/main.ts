import mermaid from "mermaid";
import initCompiler, { compile } from "wipple-visualizer";
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
const graph = document.querySelector("#graph") as HTMLDivElement;
const output = document.querySelector("#output") as HTMLDivElement;

const query = new URLSearchParams(window.location.search);
if (query.has("code")) {
    code.value = query.get("code")!;
}

const update = async () => {
    const url = new URL(window.location.href);
    url.searchParams.set("code", code.value);
    window.history.replaceState({}, "", url.toString());

    await initCompiler();

    const [outputString, graphString] = compile(code.value);

    if (graphString) {
        const { svg } = await mermaid.render("graphSvg", graphString);
        graph.innerHTML = svg;
    } else {
        graph.innerHTML = "";
    }

    output.innerHTML = ansi.ansi_to_html(outputString);
};

update();
code.addEventListener("input", debounce(300, update));
