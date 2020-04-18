import {RenderState, setup, default as load_program} from "./pkg/hello_wasm.js";

let makeInp = (cont, label, val) => {
    let inp = document.createElement("input")
    inp.name = label;
    inp.value = val;

    let lab = document.createElement("label")
    lab.htmlFor = label;
    let txt = document.createTextNode(label + ": ")
    lab.appendChild(txt);

    cont.appendChild(lab);
    cont.appendChild(inp);
    return inp
}

window.addEventListener('DOMContentLoaded', _ => {
    let screen = document.createElement("canvas")
    screen.width = 1040
    screen.height = 740
    const {width, height} = screen
    document.body.appendChild(screen);

    let job_count = document.createElement("div")
    document.body.appendChild(job_count);

    let cont = document.createElement("div")
    let x = makeInp(cont, 'x', 1)
    let y = makeInp(cont, 'y', 0)
    let z = makeInp(cont, 'z', 0)

    let button = document.createElement("button")
    button.appendChild(document.createTextNode("Update"));
    cont.appendChild(button);
    document.body.appendChild(cont);

    console.log('Loading Program')
    const ctx = screen.getContext('2d');

    load_program().then(program => {
        let state = setup(width, height);
        button.addEventListener('click', _ => {
            let x_val = parseFloat(x.value)
            let y_val = parseFloat(y.value)
            let z_val = parseFloat(z.value)
            if (isFinite(x_val) && isFinite(y_val) && isFinite(z_val)) {
                state.set_camera_to(x_val, y_val, z_val)
            }
        });

        const render = (timestamp) => {
            const start = new Date();
            state.tick();
            const pointer = state.img();
            const data = new Uint8ClampedArray(program.memory.buffer, pointer, width * height * 4)
            const img = new ImageData(data, width, height);
            ctx.putImageData(img, 0, 0);
            const end = new Date();
            job_count.innerHTML = `
            FPS: ${Math.round(1000 / (end - start))} <br />
            Number of rays: ${state.active_rays()} `;

            window.requestAnimationFrame(render);
        };
        render()
    })
})