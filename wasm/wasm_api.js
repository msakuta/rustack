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

export function wasm_set_stroke_style(str){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.strokeStyle = str;
}

export function wasm_begin_path(){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.beginPath();
}

export function wasm_move_to(x0, y0){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.moveTo(x0, y0);
}

export function wasm_line_to(x0, y0){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.lineTo(x0, y0);
}

export function wasm_stroke(){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.stroke();
}

export function wasm_rotate(angle){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.rotate(angle);
}

export function wasm_translate(x, y){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.translate(x, y);
}

export function wasm_save(){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.save();
}

export function wasm_restore(){
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext('2d');
    ctx.restore();
}
