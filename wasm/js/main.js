import { init, entry, start_step } from "../pkg/index.js";

init();

function runCommon(process) {
    // Clear output
    const output = document.getElementById("output");
    output.value = "";
    const canvas = document.getElementById("canvas");
    const canvasRect = canvas.getBoundingClientRect();
    canvas.getContext("2d").clearRect(0, 0, canvasRect.width, canvasRect.height);

    const source = document.getElementById("input").value;
    try{
        const res = process(source);
        output.value = res;
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
document.getElementById("step").addEventListener("click", () => runCommon(runStep));
document.getElementById("haltStep").addEventListener("click", () => runCommon((source) => {
    vm = null;
    updateButtonStates();
    return "Step execution halted";
}));

function runStep() {
    if (vm) {
        try {
            const stack = vm.step();
            return stack;
        }
        catch(e) {
            vm = null;
            updateButtonStates();
            return `Error: ${e}`;
        }
    }
    return "Start step execution first";
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