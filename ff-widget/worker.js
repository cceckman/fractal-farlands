// WASM worker for Fractal Overdrive.
//
import init, { Request } from './pkg/ff_widget.js';

// init() is async, so we are (in theory) not waiting to set our message handler until we've
// kicked that off.
const wasm_ready = init();

onmessage = async function(e) {
    console.log("Worker received message: ", e.data);

    // Wait for wasm init to complete before creating a WASM object (Request).
    await wasm_ready;

    let req = new Request();
    req.name = e.data.name;
    req.x = e.data.x;
    req.y = e.data.y;
    req.window = e.data.window;
    req.scale = e.data.scale;
    req.iterations = e.data.iterations;
    req.resolution = e.data.resolution;
    req.numeric = e.data.numeric;
    req.fractal = e.data.fractal;

    let data = req.render();
    postMessage(data, [data.data.buffer]);
    // postMessage("ok");
};

console.log("initialized wasm worker");
