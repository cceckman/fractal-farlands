// WASM worker for Fractal Overdrive.
//
import init, { Request } from './pkg/ff_widget.js';

// init() is async, so we are (in theory) not waiting to set our message handler until we've
// kicked that off.
const wasm_ready = init();

onmessage = async function(msg) {
    let request = msg.data;
    console.log("Worker received message: ", request);

    // Wait for wasm init to complete before creating a WASM object (Request).
    await wasm_ready;

    let req = new Request();
    // We transit the numeric fields as String rather than native BigInt,
    // as we'd need to pass through some serialization/deserialization anyway to convert to num::BigInt.
    req.name = request.name;
    req.x = request.x.toString();
    req.y = request.y.toString();
    req.window = request.window.toString();
    req.scale = request.scale.toString();
    req.iterations = request.iterations.toString();
    req.resolution = request.resolution.toString();
    req.numeric = request.numeric;
    req.fractal = request.fractal;

    let data = req.render();
    postMessage({ request: request, image: data, }, [data.data.buffer]);
    // postMessage("ok");
};

console.log("initialized wasm worker");
