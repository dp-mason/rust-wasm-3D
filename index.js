import wasmInit from "./pkg/david_rust_web_graphics.js";

const runWasm = async () => {
  const beginT = new Date();
  const beginSeconds = beginT.getTime() * 0.001;

  // Instantiate our wasm module
  const rustWasm = await wasmInit("./pkg/david_rust_web_graphics_bg.wasm");

  var wasmByteMemoryArray = new Uint8Array(rustWasm.memory.buffer);

  // Get our canvas element from our index.html
  const canvasElement = document.querySelector("canvas");

  // Set up Context and ImageData on the canvas
  const canvasContext = canvasElement.getContext("2d");
  const canvasImageData = canvasContext.createImageData(
    canvasElement.width,
    canvasElement.height
  );
  
  function renderFrame(){
    var date = new Date();
    var seconds = (date.getTime() * 0.001) - beginSeconds;
    
    var frameBufPtr;

    frameBufPtr = rustWasm.ico_anim(seconds);
    
    //console.log(seconds);

    // handles memory grow events where old lin ear mem is invalidated, seems hacky, but only solution I could find
    if (wasmByteMemoryArray === null || wasmByteMemoryArray.buffer !== rustWasm.memory.buffer) {
      wasmByteMemoryArray = new Uint8Array(rustWasm.memory.buffer);
    }


    const imageDataArray = wasmByteMemoryArray.slice(
      frameBufPtr,
      frameBufPtr + canvasElement.width * canvasElement.height * 4
    );

    // Set the values to the canvas image data
    canvasImageData.data.set(imageDataArray);

    // Clear the canvas
    canvasContext.clearRect(0, 0, canvasElement.width, canvasElement.height);

    // Put the frame buffer onto the canvas
    canvasContext.putImageData(canvasImageData, 0, 0);
  };

  setInterval(() => {
    renderFrame();
  }, 1000 / 60);
};

runWasm();