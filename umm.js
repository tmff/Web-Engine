import init from "./pkg/final_year_project.js";

function LoadApp(){
    init().then(() => {
        console.log("WASM Loaded");
    });
}

function test(){
    console.log("Test");
}

function LoadFiles(){
    document.getElementById('fileInput').addEventListener('change', function(event) {
        const files = event.target.files;
        if (files.length > 0) {
            // Assume `loadFiles` is a function exposed from Rust
            loadFiles(files).catch(console.error);
        }
    });
}