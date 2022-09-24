import { init, entry, start_step } from "../pkg/index.js";

init();

function runCommon(process, clearOutput=true) {
    // Clear output
    const output = document.getElementById("output");
    if (clearOutput) {
        output.value = "";
    }
    const canvas = document.getElementById("canvas");
    const canvasRect = canvas.getBoundingClientRect();
    canvas.getContext("2d").clearRect(0, 0, canvasRect.width, canvasRect.height);

    const source = document.getElementById("input").value;
    try{
        const res = process(source);
    }
    catch(e){
        output.value = e;
    }
}

let vm = null;

document.getElementById("run").addEventListener("click", () => runCommon(entry));
document.getElementById("startStep").addEventListener("click", () => runCommon((source) => {
    vm = start_step(source);
    updateButtonStates();
    return runStep();
}));
document.getElementById("step").addEventListener("click", () => runCommon(runStep, false));
document.getElementById("haltStep").addEventListener("click", () => runCommon((source) => {
    vm = null;
    updateButtonStates();
    return "Step execution halted";
}));

function runStep() {
    if (vm) {
        try {
            const ret = vm.step();
            const stack = vm.get_stack();
            renderStack(stack, vm.get_vars());
            return ret;
        }
        catch(e) {
            vm = null;
            updateButtonStates();
            throw e;
        }
    }
    throw "Start step execution first";
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

function renderStack(stack, varStackJson) {
    const canvas = document.getElementById("stack");
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "rgb(255, 255, 255)";
    ctx.fillRect(0, 0, 500, 500);

    ctx.fillStyle = "black";
    ctx.fillText("Stack:", stackLeft, stackTop - 5);

    for(let i in stack) {
        const val = stack[i];
        renderRect(ctx, stackLeft + i * 100, stackTop, val);
    }

    console.log(`vars: ${varStackJson}`);

    ctx.fillStyle = "black";
    ctx.fillText("Execution stack:", frameLeft, frameTop - 5);

    const varStack = JSON.parse(varStackJson);

    let offset = frameTop;
    for(let n = varStack.length - 1; 0 <= n; n--) {
        const { name, vars } = varStack[n];
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
        document.getElementById("step").removeAttribute("disabled");
        document.getElementById("haltStep").removeAttribute("disabled");
    }
    else{
        document.getElementById("step").setAttribute("disabled", "");
        document.getElementById("haltStep").setAttribute("disabled", "");
    }
}

document.getElementById("input").value = `
10 20 + puts
`;

const samples = document.getElementById("samples");

["function.txt", "fibonacci.txt", "if.txt", "recurse.txt"]
    .forEach(fileName => {
    const link = document.createElement("a");
    link.href = "#";
    link.addEventListener("click", () => {
        fetch("scripts/" + fileName)
            .then(file => file.text())
            .then(text => document.getElementById("input").value = text);
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