// WASM worker for Fractal Overdrive.
//
import init, { Request } from './pkg/ff_widget.js';

// init() is async, so we are (in theory) not waiting to set our message handler until we've
// kicked that off.
const wasm_ready = init();

let render_queue = [];

async function run_render() {
    // Wait for wasm init to complete before creating a WASM object (Request).
    await wasm_ready;

    // Handle only the latest request.
    let last = render_queue.pop();
    // Discard the rest -- with acknowledgement, but no data.
    for (const req of render_queue) {
        postMessage({ request: req, image: null });
    }
    render_queue = [];

    // Our last-received request doesn't match our last-sent request.
    // Render this one.

    let req = new Request();
    // We transit the numeric fields as String rather than native BigInt,
    // as we'd need to pass through some serialization/deserialization anyway to convert to num::BigInt.
    req.name = last.name;
    req.x = last.x.toString();
    req.y = last.y.toString();
    req.half_window = last.halfWindow.toString();
    req.scale = last.scale.toString();
    req.iterations = last.iterations.toString();
    req.resolution = last.resolution.toString();
    req.numeric = last.numeric;
    req.fractal = last.fractal;
    let data = req.render();
    postMessage({ request: last, image: data, }, [data.data.buffer]);
}

onmessage = function(msg) {
    render_queue.push(msg.data);
    console.log("Worker received message: ", msg.data);
    // Run render asynchronously, so if we have multiple messages queued we can flush them out.
    run_render()
};

console.log("initialized wasm worker");
