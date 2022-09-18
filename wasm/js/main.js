import { entry } from "../pkg/index.js";


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

document.getElementById("run").addEventListener("click", () => runCommon(entry));

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
