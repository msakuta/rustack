// Wait for the html to fully load before loading main.js, since it assumes static html elements exist.
window.onload = () => {
    import("./main.js").catch(console.error);
}
