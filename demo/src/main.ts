import { instance as vizInstance } from "@viz-js/viz";
import "./style.css";
import { compile } from "wipple-compiler";
import DOMPurify from "dompurify";
import { marked } from "marked";

const viz = await vizInstance();
console.log(viz);

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

    const [syntaxError, graphString, tys, feedback] = compile(code.value);

    if (syntaxError) {
        log.innerText = `Syntax error: ${syntaxError}`;
    } else {
        try {
            graph.children[0]?.remove();
            graph.appendChild(viz.renderSVGElement(graphString));
        } catch (e) {
            console.error(e);
        }

        const markdown = DOMPurify.sanitize(marked.parse(feedback, { async: false }));
        log.innerHTML = `
            <div class="markdown-body">${markdown}</div>
            <pre>${tys}</pre>
        `;
    }
};

update();
code.addEventListener("input", debounce(500, update));
