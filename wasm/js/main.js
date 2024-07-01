import { init, entry, start_step } from "../pkg/index.js";

init();

function runCommon(process, clearOutput=true, measureTime=true) {
    // Clear output
    const output = document.getElementById("output");
    if (clearOutput) {
        output.value = "";
        const canvas = document.getElementById("canvas");
        const canvasRect = canvas.getBoundingClientRect();
        const ctx = canvas.getContext("2d");
        ctx.clearRect(0, 0, canvasRect.width, canvasRect.height);
        ctx.reset();
        document.getElementById("timeMessage").innerHTML = "";
    }

    const source = document.getElementById("input").value;
    try{
        const start = performance.now();
        const res = process(source);
        const end = performance.now();
        if (measureTime) {
            document.getElementById("timeMessage").innerHTML = `Execution time: ${(end - start).toFixed(1)} ms (See <a href="#Time">notes</a>)`;
        }
    }
    catch(e){
        output.value = e;
    }
}

let vm = null;
let sourceText = "";

document.getElementById("run").addEventListener("click", () => runCommon(entry));
document.getElementById("startStep").addEventListener("click", () => runCommon((source) => {
    vm = start_step(source);
    sourceText = source;
    document.getElementById("fixedInput").innerHTML = source;
    updateButtonStates();
    return runStep();
}, true, false));
document.getElementById("startAutoStep").addEventListener("click", () => runCommon((source) => {
    vm = start_step(source);
    sourceText = source;
    document.getElementById("fixedInput").innerHTML = source;
    updateButtonStates();
    runAutoStep();
}, true, false));
document.getElementById("step").addEventListener("click", () => runCommon(runStep, false));
document.getElementById("haltStep").addEventListener("click", () => runCommon((source) => {
    vm = null;
    updateButtonStates();
    return "Step execution halted";
}, true, false));
document.getElementById("clearCanvas").addEventListener("click", () => {
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext("2d");
    ctx.reset();
    ctx.fillStyle = "rgb(255, 255, 255)";
    ctx.clearRect(0, 0, canvas.width, canvas.height);
});

function runStep() {
    if (vm) {
        try {
            const ret = vm.step();
            const first = sourceText.substring(0, ret[0]);
            const middle = sourceText.substring(ret[0], ret[1]);
            const last = sourceText.substring(ret[1]);
            document.getElementById("fixedInput").innerHTML = first + `<span style="color: red; background-color: cyan">${middle}</span>` + last;
            const stack = vm.get_stack();
            renderStack(stack, vm.get_exec_stack());
            return "";
        }
        catch(e) {
            vm = null;
            updateButtonStates();
            throw e;
        }
    }
    throw "Start step execution first";
}

function runAutoStep() {
    if (vm) {
        runStep();
        requestAnimationFrame(runAutoStep, 50);
    }
}

const stackTop = 50;
const stackLeft = 20;
const frameTop = stackTop + 50;
const frameLeft = 20;
const frameMargin = 10;
const frameNameHeight = 25;
const varLeft = stackLeft + frameMargin;
const varWidth = 100;
const varHeight = 30;

function estimateHeight(stack, execStack) {
    let offset = frameTop;
    for(let n = execStack.length - 1; 0 <= n; n--) {
        offset += execStack[n].vars.length * varHeight + frameMargin * 3 + frameNameHeight;
    }
    return offset;
}

function renderStack(stack, execStackJson) {
    const execStack = JSON.parse(execStackJson);

    const canvas = document.getElementById("stack");
    canvas.height = estimateHeight(stack, execStack);
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "rgb(255, 255, 255)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = "black";
    ctx.fillText(`Operand stack[${stack.length}]: `, stackLeft, stackTop - 5);

    for(let i in stack) {
        const val = stack[i];
        renderRect(ctx, stackLeft + i * 100, stackTop, val);
    }

    ctx.fillStyle = "black";
    ctx.fillText(`Execution stack[${execStack.length}]:`, frameLeft, frameTop - 5);

    let offset = frameTop;
    for(let n = execStack.length - 1; 0 <= n; n--) {
        const { name, vars } = execStack[n];
        const varTop = offset + frameMargin + frameNameHeight;

        ctx.beginPath();
        ctx.rect(frameLeft, offset, varWidth * 2 + frameMargin * 2, varHeight * vars.length + frameMargin * 2 + frameNameHeight);
        ctx.fillStyle = "rgb(191, 191, 191)";
        ctx.fill();
        ctx.strokeStyle = "black";
        ctx.stroke();

        ctx.fillStyle = "black";
        ctx.fillText(`function: ${name}`, varLeft, varTop - 20);

        ctx.fillText(`variables:`, varLeft, varTop - 5);

        for(let i = 0; i < vars.length; i++) {
            const [key, value] = vars[i];
            renderRect(ctx, varLeft, varTop + varHeight * i, key);
            renderRect(ctx, varLeft + varWidth, varTop + varHeight * i, value, "rgb(127, 255, 255)");
        }
        offset += vars.length * varHeight + frameMargin * 3 + frameNameHeight;
    }
}

function renderRect(ctx, x, y, txt, color="rgb(127, 255, 127)") {
    ctx.beginPath();
    ctx.rect(x, y, 100, varHeight);
    ctx.fillStyle = color;
    ctx.fill();
    ctx.strokeStyle = "black";
    ctx.stroke();

    ctx.fillStyle = "black";
    ctx.fillText(txt, x + 5, y + varHeight - 5);
}

function updateButtonStates() {
    if(vm){
        document.getElementById("code").style.display = "none";
        document.getElementById("fixedCode").style.display = "block";
        document.getElementById("run").setAttribute("disabled", "");
        document.getElementById("startStep").setAttribute("disabled", "");
        document.getElementById("step").removeAttribute("disabled");
        document.getElementById("haltStep").removeAttribute("disabled");
    }
    else{
        document.getElementById("code").style.display = "block";
        document.getElementById("fixedCode").style.display = "none";
        document.getElementById("run").removeAttribute("disabled");
        document.getElementById("startStep").removeAttribute("disabled");
        document.getElementById("step").setAttribute("disabled", "");
        document.getElementById("haltStep").setAttribute("disabled", "");
    }
}

document.getElementById("input").value = `
10 20 + puts
`;

const samples = document.getElementById("samples");

["function.txt", "fibonacci.txt", "if.txt", "for.txt", "recurse.txt", "canvas.txt", "koch.txt", "mandel_canvas.txt", "mandel_canvas_recursive.txt"]
    .forEach(fileName => {
        const link = document.createElement("a");
        link.href = "#";
        link.addEventListener("click", () => {
            fetch("scripts/" + fileName)
                .then(file => file.text())
                .then(text => {
                    document.getElementById("input").value = text
                    vm = null;
                    updateButtonStates();
                });
    });
    link.innerHTML = fileName;
    samples.appendChild(link);
    samples.append(" ");
})

// function updateStackSvg() {
//     const stack = document.getElementById("stack");
//     while(stack.firstChild)
//         stack.removeChild(stack.firstChild);

//     for()
//     stack.appendChild("");
// }