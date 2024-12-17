import init, { numeric_options, Request } from './pkg/ff_widget.js';

let init_done = init();

let widget_count = 0;

class OverdriveElement extends HTMLElement {
    constructor() {
        super()
        let template = document.getElementById("fractal-overdrive-tmpl");
        let templateContent = template.content;
        let shadowRoot = this.attachShadow({ mode: "open" });
        shadowRoot.appendChild(templateContent.cloneNode(true));

        this.x = this.shadowRoot.querySelector("#input-x");
        this.y = this.shadowRoot.querySelector("#input-y");
        this.window = this.shadowRoot.querySelector("#input-window");
        this.scale = this.shadowRoot.querySelector("#input-scale");
        this.iterations = this.shadowRoot.querySelector("#input-iterations");
        this.resolution = this.shadowRoot.querySelector("#input-resolution");
        this.numeric = this.shadowRoot.querySelector("#input-numeric");
        this.fractal = this.shadowRoot.querySelector("#input-fractal");
        this.go = this.shadowRoot.querySelector("#input-go");
        this.canvas = this.shadowRoot.querySelector("canvas");

        this.name = `fractal-overdrive ${widget_count}`;
        widget_count++;
        this.worker = null;
    }

    updateOptions() {
        let opts = numeric_options(this.fractal.value);
        let selected = this.numeric.value;
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
    }

    connectedCallback() {
        console.log("connected")

        // Start up the new thread:
        this.worker = new Worker("./worker.js", { type: "module" });
        this.worker.addEventListener("message", (msg) => { this.getNewData(msg); });

        // Initialize from named parameters:
        const fractal = this.attributes.getNamedItem("fractal")?.value;
        if (fractal) {
            let item = this.fractal.namedItem(fractal);
            if (item) {
                item.selected = true;
            }
        }
        this.fractal.addEventListener("change", () => { this.updateOptions() });

        const numeric = this.attributes.getNamedItem("numeric")?.value;
        if (numeric) {
            let item = this.numeric.namedItem(numeric);
            if (item) {
                item.selected = true;
            }
        }

        const x = this.attributes.getNamedItem("x")?.value ?? 0;
        const y = this.attributes.getNamedItem("y")?.value ?? 0;
        const window = this.attributes.getNamedItem("window")?.value ?? 4;
        const scale = this.attributes.getNamedItem("scale")?.value ?? 1;
        const resolution = this.attributes.getNamedItem("resolution")?.value ?? 256;
        const iterations = this.attributes.getNamedItem("iterations")?.value ?? 16;
        const pairs = [[x, this.x], [y, this.y], [window, this.window], [scale, this.scale], [resolution, this.resolution], [iterations, this.iterations]];
        for (const [value, elem] of pairs) {
            elem.value = value;
        }

        let name = this.attributes.getNamedItem("name")?.value;
        if (name) {
            this.name = name;
        }

        this.canvas.height = resolution;
        this.canvas.width = resolution;

        // Chain of initializations:
        init_done.then(() => { this.wasmInit() });

        // - Read specified attributes for defaults
        // - Fill in defaults on shadow DOM nodes
    }

    disconnectedCallback() {
        this.worker.terminate();
        this.worker = null;
    }

    wasmInit() {
        console.log("init done; starting WASM for ", this.name, this.shadowRoot.getRootNode());
        this.updateOptions();
        this.shadowRoot.querySelector("form").addEventListener("submit", (e) => {
            console.log("got submit event");
            e.preventDefault();
            this.render();
        });
        this.go.disabled = false;
        // We set up the receiving end, from the worker, during connectCallback.

        // Perform an initial re-render:
        this.worker.postMessage(this.makeRequest());
    }

    // Construct the request object to send to the WASM worker.
    makeRequest() {
        // Use a raw Object, as WASM objects are not transferable:
        // https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage
        return {
            name: this.name,
            x: this.x.value,
            y: this.y.value,
            window: this.window.value,
            scale: this.scale.value,
            iterations: this.iterations.value,
            resolution: this.resolution.value,
            numeric: this.numeric.value,
            fractal: this.fractal.value,
        }
    }

    // Called on submit, to handle re-rendering.
    render() {
        this.worker.postMessage(this.makeRequest());
    }

    getNewData(msg) {
        console.log("got new message from worker in main: ", msg);
    }

}

window.customElements.define("fractal-overdrive", OverdriveElement);

await init() 
