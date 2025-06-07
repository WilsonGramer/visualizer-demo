import * as d3graphviz from "d3-graphviz";
import "./style.css";
import { compile } from "wipple-compiler";

const debounce = (timeout: number, f: () => void) => {
    let timeoutId: number | undefined;
    return () => {
        window.clearTimeout(timeoutId);
        timeoutId = window.setTimeout(f, timeout);
    };
};

const code = document.querySelector("#code") as HTMLTextAreaElement;
const graph = document.querySelector("#graph") as HTMLDivElement;
const log = document.querySelector("#log") as HTMLDivElement;

const query = new URLSearchParams(window.location.search);
if (query.has("code")) {
    code.value = query.get("code")!;
}

const graphviz = d3graphviz.graphviz(graph, {
    width: graph.clientWidth,
    height: graph.clientHeight,
    fit: true,
});

const update = async () => {
    const url = new URL(window.location.href);
    url.searchParams.set("code", code.value);
    window.history.replaceState({}, "", url.toString());

    const [syntaxError, graph, tys] = compile(code.value);

    try {
        graphviz.renderDot(graph);
    } catch (e) {
        console.error(e);
    }

    log.innerText = syntaxError + tys;
};

update();
code.addEventListener("input", debounce(500, update));
