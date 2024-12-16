import init, { attach } from './pkg/ff_widget.js';

async function run() {
    await init();

    for (const el of document.querySelectorAll(".fractal-overdrive")) {
        attach(el);
    }
}

run()
