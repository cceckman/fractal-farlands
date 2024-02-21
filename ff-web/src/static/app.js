
class AreaParams {
    constructor() {
        // Query parameters, then defaults
        // TODO: Query parameters, then area parameters, then defaults
        const queryParams = new URLSearchParams(window.location.search);
        this.x = BigInt(queryParams.get("x") ?? 0);
        this.y = BigInt(queryParams.get("y") ?? 0);
        this.window = BigInt(queryParams.get("window") ?? 4);
        this.scale = BigInt(queryParams.get("scale") ?? 1);
    }

    // Pan towards (x, y) at the current scale.
    pan(offsetX, offsetY) {
        // Figure out window size that was clicked in:
        const image = Array.from(document.getElementsByClassName("img-fractal"))[0];
        const width = image.width;
        const height = image.height;
        console.assert(width == height);
        const res = BigInt(width);

        // We have a vector from corner-to-click, but we want the vector from center-to-click.
        // I was confused for a while as to why these are backwards of each other, but it makes sense
        // if considering them both as vectors from the center.
        //
        // Consider the vector from the origin (center) to the upper-left corner:
        // the upper-left corner has coordinates (-width/2, height/2), because the upper-left is in the second quadrant.
        // And in this space, the vector we have - "corner-to-click", has value (offsetX, -offsetY).
        // Add these vectors together, and it looks like this:
        let vecX = BigInt(offsetX - width / 2);
        let vecY = BigInt(height / 2 - offsetY);

        // Above is an integer.
        // vecX / res is a rational, representing what portion of the window we should move, in units of fractional windows.
        console.log(`move: ${vecX} / ${res}, ${vecY} / ${res}`);
        // vecX / res * (window / scale) is a rational, representing what portion of the area is traversed, in units of actual units.
        // We get that unit by updating the numerator:
        vecX *= this.window;
        vecY *= this.window;
        // We imagine a denominator "scale*res", but we don't have to track it-
        // until we start doing math with these other units.
        // Then, we need to make them all over scale*res, by multiplying the denominator:
        this.scale *= res;
        // And numerators:
        this.x *= res;
        this.y *= res;
        this.window *= res;

        // Our vecX and vecY are the vectors from our old center to our new center.
        // ...again, somehow our Y sign is backwards.
        this.x += vecX;
        this.y -= vecY;
    }

    zoom(numerator, denominator) {
        this.scale *= BigInt(denominator);
        this.window *= BigInt(numerator)
        // Preserve the center:
        this.x *= BigInt(denominator);
        this.y *= BigInt(denominator);
    }
}

/// Update the form to the new params, and submit it
function jump(params) {
    console.log("new parameters: ", params);
    document.getElementById("input-x").value = params.x.toString();
    document.getElementById("input-y").value = params.y.toString();
    document.getElementById("input-window").value = params.window.toString();
    document.getElementById("input-scale").value = params.scale.toString();
    document.getElementById("input-scale").value = params.scale.toString();
    document.getElementById("form-rerender").submit();
}

/// Onclick handler for image panes.
function click_to_zoom_cb(/*PointerEvent*/ev) {
    console.log(`got pointer event at ${ev.offsetX}, ${ev.offsetY} `, ev);
    let params = new AreaParams();
    params.pan(ev.offsetX, ev.offsetY);
    jump(params);
}

function zoom_in_cb(ev) {
    console.log("got zoom in: ", ev);
    let params = new AreaParams();
    params.zoom(2, 3);
    jump(params);
}
function zoom_out_cb(ev) {
    console.log("got zoom out: ", ev);
    let params = new AreaParams();
    params.zoom(3, 2);
    jump(params);
}

/// Setup function: attach click etc. callbacks
function attach_callbacks() {
    const images = Array.from(document.getElementsByClassName("img-fractal"));
    for (const img of images) {
        // https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events: touch-aware alternative to "click"
        img.addEventListener("pointerup", click_to_zoom_cb);
    }
    document.getElementById("button-in").addEventListener("pointerup", zoom_in_cb);
    document.getElementById("button-out").addEventListener("pointerup", zoom_out_cb);
}

attach_callbacks();
