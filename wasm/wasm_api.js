export function wasm_print(str){
    document.getElementById("output").value += str;
}

export function wasm_set_fill_style(str){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = str;
}

export function wasm_rectangle(x0, y0, x1, y1){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.fillRect(x0, y0, x1, y1);
}