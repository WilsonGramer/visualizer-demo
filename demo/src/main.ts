import mermaid from "mermaid";
import initCompiler, { compile } from "wipple-compiler";
import DOMPurify from "dompurify";
import { marked } from "marked";
import "./style.css";

mermaid.initialize({ startOnLoad: false });

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

const update = async () => {
    const url = new URL(window.location.href);
    url.searchParams.set("code", code.value);
    window.history.replaceState({}, "", url.toString());

    await initCompiler();

    const [syntaxError, graphString, tys, feedback] = compile(code.value);

    if (syntaxError) {
        log.innerText = `Syntax error: ${syntaxError}`;
    } else {
        const { svg } = await mermaid.render("graphSvg", graphString);
        graph.innerHTML = svg;

        const markdown = DOMPurify.sanitize(marked.parse(feedback, { async: false }));
        log.innerHTML = `
            <div class="markdown-body">${markdown}</div>
            <pre>${tys}</pre>
        `;
    }
};

update();
code.addEventListener("input", debounce(500, update));
