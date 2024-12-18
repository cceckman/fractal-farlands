import init, { reduce, numeric_options } from './pkg/ff_widget.js';

let init_done = init();

let widget_count = 0;

class OverdriveElement extends HTMLElement {
    constructor() {
        super()
        let template = document.getElementById("fractal-overdrive-tmpl");
        let templatevalue = template.content;
        let shadowRoot = this.attachShadow({ mode: "open" });
        shadowRoot.appendChild(templatevalue.cloneNode(true));

        this.name = `fractal-overdrive ${widget_count}`;
        widget_count++;
        this.worker = null;
        this.outstandingCount = 0;

        this.xElement = this.shadowRoot.querySelector("#input-x");
        this.yElement = this.shadowRoot.querySelector("#input-y");
        this.windowElement = this.shadowRoot.querySelector("#input-window");
        this.scaleElement = this.shadowRoot.querySelector("#input-scale");
        this.iterationsElement = this.shadowRoot.querySelector("#input-iterations");
        this.resolutionElement = this.shadowRoot.querySelector("#input-resolution");
        this.numericElement = this.shadowRoot.querySelector("#input-numeric");
        this.fractalElement = this.shadowRoot.querySelector("#input-fractal");

        this.statusElement = this.shadowRoot.querySelector("#status");

        this.canvasElement = this.shadowRoot.querySelector("canvas");
        this.currentDisplay = null;
        this.hasOriginal = false;
    }

    updateOptions() {
        let opts = numeric_options(this.fractalElement.value);
        let selected = this.numericElement.value;
        for (let i = this.numericElement.length - 1; i >= 0; i--) {
            this.numericElement.remove(i);
        }
        for (const o of opts) {
            let el = document.createElement("option");
            el.value = o;
            el.text = o;
            el.id = o;
            this.numericElement.add(el);
        }
        this.numericElement.value = selected;
        if (!this.numericElement.value) {
            const numericElement = this.attributes.getNamedItem("numeric")?.value ?? "f32";
            this.numericElement.value = numeric;
        }
    }

    reset() {
        // Initialize the data fields from attributes:
        this.xElement.value = this.attributes.getNamedItem("x")?.value ?? 0;
        this.yElement.value = this.attributes.getNamedItem("y")?.value ?? 0;
        this.windowElement.value = this.attributes.getNamedItem("window")?.value ?? 2;
        this.scaleElement.value = this.attributes.getNamedItem("scale")?.value ?? 1;
        let resolution = this.attributes.getNamedItem("resolution")?.value ?? 256;
        this.resolutionElement.value = resolution;
        this.iterationsElement.value = this.attributes.getNamedItem("iterations")?.value ?? 16;

        const fractal = this.attributes.getNamedItem("fractal")?.value ?? "mandelbrot";
        this.fractalElement.value = fractal;
        // Insert this option before we load the actual options:
        {
            const numeric = this.attributes.getNamedItem("numeric")?.value ?? "f32";
            let el = document.createElement("option");
            el.value = numeric;
            el.text = numeric;
            el.id = numeric;
            this.numericElement.add(el);
            this.numericElement.value = numeric;
        }
        // We have to defer "the actual options" until we've instantiated the WASM module locally.
        this.canvasElement.height = resolution;
        this.canvasElement.width = resolution;
    }

    connectedCallback() {
        console.log("connected");

        for (const input of this.querySelectorAll("input")) {
            input.disabled = true;
        }

        const name = this.attributes.getNamedItem("name")?.value;
        if (name) {
            this.name = name;
        }
        // Reset the inputs to match the defaults:
        this.reset();
        let resolution = parseInt(this.resolutionElement.value);

        // Resize, now and continuously:
        for (const el of [this.xElement, this.yElement, this.windowElement, this.scaleElement, this.resolutionElement, this.iterationsElement]) {
            el.size = el.value.length;
            el.addEventListener("change", (_ev) => {
                el.size = el.value.length;
                this.updateStatus();
            });
        }

        // Set up the "original" image, in case we don't interact.
        let original_request = this.makeRequest();
        let original = this.attributes.getNamedItem("original");
        if (original) {
            this.hasOriginal = true;
            let image = new Image(); // creates an HTMLImageElement!
            image.src = original.value;
            this.outstandingCount += 1;
            image.decode().then(() => {
                this.outstandingCount -= 1;
                let same = this.requestIsCurrent(original_request);
                if (same && !this.currentDisplay) {
                    let xscale = resolution / image.naturalWidth;
                    let yscale = resolution / image.naturalHeight;
                    // Nothing already rendered, and we haven't moved.
                    // Display the image.
                    this.currentDisplay = original_request;
                    let context = this.canvasElement.getContext("2d");
                    context.scale(xscale, yscale);
                    context.drawImage(image, 0, 0);
                }
                this.updateStatus();
            }).catch((err) => {
                console.log("error displaying default image for", this.name, ": ", err);
            });
        }

        // Finally, kick off the WASM worker:

        this.worker = new Worker("./fractal-overdrive-worker.js", { type: "module" });
        this.worker.addEventListener("message", (msg) => { this.getNewData(msg); });

        // Chain of initializations:
        init_done.then(() => { this.wasmInit() });
    }

    updateStatus() {
        if (this.outstandingCount != 0) {
            this.statusElement.classList.add("loader");
            this.statusElement.innerHTML = "";
            return
        }
        // All oustanding background work is complete.
        this.statusElement.classList.remove("loader");
        if (this.requestIsCurrent(this.currentDisplay)) {
            this.statusElement.innerText = "✓";
        } else {
            this.statusElement.innerText = "!";
        }
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

    // Sync back the "last-rendered" to the view fields.
    syncToDisplay() {
        if (!this.currentDisplay) {
            return;
        }
        this.xElement.value = this.currentDisplay.x;
        this.yElement.value = this.currentDisplay.y;
        this.scaleElement.value = this.currentDisplay.scale;
        this.windowElement.value = this.currentDisplay.halfWindow;
        this.resolutionElement.value = this.currentDisplay.resolution;
        this.iterationsElement.value = this.currentDisplay.iterations;
        this.numericElement.value = this.currentDisplay.numeric;
        this.fractalElement.value = this.currentDisplay.fractal;
        this.reduce();
    }

    wasmInit() {
        this.fractalElement.addEventListener("change", () => {
            this.updateOptions();
            this.updateStatus();
        });
        this.numericElement.addEventListener("change", () => {
            this.updateStatus();
        });
        this.updateOptions();

        this.shadowRoot.querySelector("#out").addEventListener("click", (ev) => {
            ev.preventDefault();
            this.syncToDisplay();
            let window = BigInt(this.windowElement.value);
            let new_window = window * BigInt(2);
            this.windowElement.value = new_window;
            this.reduce();
            this.render();
        });
        this.shadowRoot.querySelector("#in").addEventListener("click", (ev) => {
            ev.preventDefault();
            this.syncToDisplay();
            let x = BigInt(this.xElement.value);
            let y = BigInt(this.yElement.value);
            let scale = BigInt(this.scaleElement.value);
            const two = BigInt(2);
            x *= two;
            y *= two;
            scale *= two;
            this.xElement.value = x;
            this.yElement.value = y;
            this.scaleElement.value = scale;
            this.reduce();
            this.render();
        });

        this.shadowRoot.querySelector("#up").addEventListener("click", (ev) => {
            ev.preventDefault();
            this.syncToDisplay();
            let y = parseInt(this.yElement.value);
            let step = parseInt(this.windowElement.value);
            let new_y = y + (step);
            if (!isNaN(new_y)) {
                this.yElement.value = new_y.toString();
                this.render();
            }
        });
        this.shadowRoot.querySelector("#down").addEventListener("click", (ev) => {
            ev.preventDefault();
            this.syncToDisplay();
            let y = parseInt(this.yElement.value);
            let step = parseInt(this.windowElement.value);
            let new_y = y - (step);
            if (!isNaN(new_y)) {
                this.yElement.value = new_y.toString();
                this.render();
            }
        });
        this.shadowRoot.querySelector("#right").addEventListener("click", (ev) => {
            ev.preventDefault();
            this.syncToDisplay();
            let x = parseInt(this.xElement.value);
            let step = parseInt(this.windowElement.value);
            let new_x = x + (step);
            if (!isNaN(new_x)) {
                this.xElement.value = new_x.toString();
                this.render();
            }
        });
        this.shadowRoot.querySelector("#left").addEventListener("click", (ev) => {
            ev.preventDefault();
            this.syncToDisplay();
            let x = parseInt(this.xElement.value);
            let step = parseInt(this.windowElement.value);
            let new_x = x - (step);
            if (!isNaN(new_x)) {
                this.xElement.value = new_x.toString();
                this.render();
            }
        });

        this.shadowRoot.querySelector("#reset")
            .addEventListener("click", (ev) => {
                ev.preventDefault();
                this.reset();
                this.render();
            });
        this.shadowRoot.querySelector("#update")
            .addEventListener("click", (ev) => {
                ev.preventDefault();
                this.render();
            });

        for (const input of this.querySelectorAll("input")) {
            input.disabled = false;
        }

        // Only re-render client-side if there's no default.
        if (!this.hasOriginal) {
            this.render();
        }
    }

    // Construct the request object to send to the WASM worker.
    makeRequest() {
        // Use a raw Object, as WASM objects are not transferable:
        // https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage
        return {
            name: this.name,
            x: this.xElement.value,
            y: this.yElement.value,
            halfWindow: this.windowElement.value,
            scale: this.scaleElement.value,
            iterations: this.iterationsElement.value,
            resolution: this.resolutionElement.value,
            numeric: this.numericElement.value,
            fractal: this.fractalElement.value,
        }
    }

    // Called on submit, to handle re-rendering.
    render() {
        console.log("starting render of", this.name);
        this.outstandingCount += 1;
        // Gray out while loading:
        let ctx = this.canvasElement.getContext("2d");
        ctx.fillStyle = "rgba(0.5, 0.5, 0.5, 0.5)";
        ctx.fillRect(0, 0, this.canvasElement.width, this.canvasElement.height);
        this.updateStatus();
        this.worker.postMessage(this.makeRequest());
    }

    /// Reduce the fractions, where we can.
    reduce() {
        let denom = BigInt(reduce(this.xElement.value, this.yElement.value, this.scaleElement.value, this.windowElement.value));

        for (const el of [this.xElement, this.yElement, this.windowElement, this.scaleElement]) {
            let v = BigInt(el.value);
            v /= denom;
            el.value = v.toString();
        }
    }

    getNewData(msg) {
        this.outstandingCount -= 1;
        let { request: original_request, image: image } = msg.data;
        if (!this.requestIsCurrent(original_request) || !image) {
            console.log("got stale render response for", this.name);
            return;
        }
        console.log("got up-to-date response for", this.name);

        this.canvasElement.width = original_request.resolution;
        this.canvasElement.height = original_request.resolution;

        let ctx = this.canvasElement.getContext("2d");
        ctx.putImageData(image, 0, 0);
        this.currentDisplay = original_request;
        this.updateStatus();
    }

}

window.customElements.define("fractal-overdrive", OverdriveElement);

await init() 
