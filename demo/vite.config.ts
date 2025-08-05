import process from "node:process";
import path from "node:path";
import { defineConfig, searchForWorkspaceRoot } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
    plugins: [tailwindcss()],
    server: {
        fs: {
            allow: [
                searchForWorkspaceRoot(process.cwd()),
                path.resolve(__dirname, "../visualizer/pkg"),
            ],
        },
    },
});
