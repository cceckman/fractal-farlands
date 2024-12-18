import init, { numeric_options } from './pkg/ff_widget.js';

let init_done = init();

let widget_count = 0;

class OverdriveElement extends HTMLElement {
    constructor() {
        super()
        let template = document.getElementById("fractal-overdrive-tmpl");
        let templateContent = template.content;
        let shadowRoot = this.attachShadow({ mode: "open" });
        shadowRoot.appendChild(templateContent.cloneNode(true));

        this.name = `fractal-overdrive ${widget_count}`;
        widget_count++;
        this.worker = null;

        this.x = undefined;
        this.y = undefined;
        this.window = undefined;
        this.scale = undefined;
        this.iterations = undefined;
        this.resolution = undefined;
        this.numeric = undefined;
        this.fractal = undefined;
        this.canvas = this.shadowRoot.querySelector("canvas");
        this.currentDisplay = null;
        this.hasOriginal = false;
    }

    updateOptions() {
        // TODO: Revise with new inputs
        return;
        let opts = numeric_options(this.fractal);
        let selected = this.numeric;
        for (let i = this.numeric.length - 1; i >= 0; i--) {
            this.numeric.remove(i);
        }
        for (const o of opts) {
            let el = document.createElement("option");
            el.value = o;
            el.text = o;
            el.id = o;
            this.numeric.add(el);
        }
        this.numeric.value = selected;
        if (!this.numeric.value) {
            const numeric = this.attributes.getNamedItem("numeric")?.value ?? "f32";
            this.numeric.value = numeric;
        }
    }

    connectedCallback() {
        console.log("connected")

        const name = this.attributes.getNamedItem("name")?.value;
        if (name) {
            this.name = name;
        }

        // Initialize the data fields from attributes:
        this.x = this.attributes.getNamedItem("x")?.value ?? 0;
        this.y = this.attributes.getNamedItem("y")?.value ?? 0;
        this.window = this.attributes.getNamedItem("window")?.value ?? 4;
        this.scale = this.attributes.getNamedItem("scale")?.value ?? 1;
        this.resolution = this.attributes.getNamedItem("resolution")?.value ?? 256;
        this.iterations = this.attributes.getNamedItem("iterations")?.value ?? 16;

        // TODO: Update "current display" portion of window
        this.fractal = this.attributes.getNamedItem("fractal")?.value ?? "mandelbrot";
        this.numeric = this.attributes.getNamedItem("numeric")?.value ?? "f32";
        /*
        // Insert this option before we load the actual options:
        {
            let el = document.createElement("option");
            el.value = numeric;
            el.text = numeric;
            el.id = numeric;
            this.numeric.add(el);
            this.numeric.selected = numeric;
        }*/
        // We have to defer "the actual options" until we've instantiated the WASM module locally.
        this.canvas.height = this.resolution;
        this.canvas.width = this.resolution;

        // Set up the "original" image, in case we don't interact.
        let original_request = this.makeRequest();
        let original = this.attributes.getNamedItem("original");
        if (original) {
            this.hasOriginal = true;
            let image = new Image(); // creates an HTMLImageElement!
            image.src = original.value;
            image.decode().then(() => {
                let same = this.requestIsCurrent(original_request);
                if (same && !this.currentDisplay) {
                    let xscale = this.resolution / image.naturalWidth;
                    let yscale = this.resolution / image.naturalHeight;
                    // Nothing already rendered, and we haven't moved.
                    // Display the image.
                    this.currentDisplay = original_request;
                    let context = this.canvas.getContext("2d");
                    context.scale(xscale, yscale);
                    context.drawImage(image, 0, 0);
                }
            }).catch((err) => {
                console.log("error displaying default image for", this.name, ": ", err);
            });
        }

        // Finally, kick off the WASM worker:

        this.worker = new Worker("./worker.js", { type: "module" });
        this.worker.addEventListener("message", (msg) => { this.getNewData(msg); });

        // Chain of initializations:
        init_done.then(() => { this.wasmInit() });
    }

    requestIsCurrent(original_request) {
        let current_want = this.makeRequest();
        let same = Object.entries(current_want).every(([key, value]) =>
            original_request[key] === value
        );
        return same;
    }

    disconnectedCallback() {
        this.worker.terminate();
        this.worker = null;
    }

    wasmInit() {
        // TODO: properly render stuff
        // this.fractal.addEventListener("change", () => { this.updateOptions() });
        this.updateOptions();
        //this.shadowRoot.querySelector("form").addEventListener("submit", (e) => {
        //    e.preventDefault();
        //    this.render();
        //});
        // this.go.disabled = false;
        // We set up the receiving end, from the worker, during connectCallback.

        // Only re-render client-side if there's no default.
        if (!this.hasOriginal) {
            this.worker.postMessage(this.makeRequest());
        }
    }

    // Construct the request object to send to the WASM worker.
    makeRequest() {
        // Use a raw Object, as WASM objects are not transferable:
        // https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage
        return {
            name: this.name,
            x: this.x,
            y: this.y,
            window: this.window,
            scale: this.scale,
            iterations: this.iterations,
            resolution: this.resolution,
            numeric: this.numeric,
            fractal: this.fractal,
        }
    }

    // Called on submit, to handle re-rendering.
    render() {
        console.log("starting render of", this.name);
        // TODO: Add a spinner here!
        this.worker.postMessage(this.makeRequest());
    }

    getNewData(msg) {
        let { request: original_request, image: image } = msg.data;
        if (!this.requestIsCurrent(original_request)) {
            console.log("got stale render response");
            return;
        }

        let ctx = this.canvas.getContext("2d");
        ctx.putImageData(image, 0, 0);
        this.currentDisplay = original_request;
    }

}

window.customElements.define("fractal-overdrive", OverdriveElement);

await init() 
