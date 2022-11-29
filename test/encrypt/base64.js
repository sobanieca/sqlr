let encoder = new TextEncoder();
let decoder = new TextDecoder();

let arg = Uint8Array.from([
     60,  28,  56, 239, 191, 189, 239, 191, 189,  98,  91, 126,
    239, 191, 189,  65, 120, 221, 169, 239, 191, 189, 121, 239,
    191, 189,   8, 239, 191, 189,  55,  13, 239, 191, 189, 239,
    191, 189,  19, 119,  45, 239, 191, 189, 115, 239, 191, 189,
     17, 239, 191, 189,  66,  83,  38, 239, 191, 189,  16, 239,
    191, 189, 239, 191, 189,  98,   0, 239, 191, 189, 239, 191,
    189, 239, 191, 189,  65, 110, 216, 190
]);

console.log(arg.buffer);

let result = decoder.decode(arg.buffer);
result = encodeURIComponent(result);
result = btoa(result);
console.log(result);

let decoded = atob(result);
decoded = decodeURIComponent(decoded);
decoded = encoder.encode(decoded);
console.log(decoded);


