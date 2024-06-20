// import the generated file.
import init from "./pkg/wasm_impl.js";
import { send_message } from "./pkg/wasm_impl.js";
init();

function call_message(msg) {
  return new Promise((resolve, _) => send_message(msg, resolve));
}

(() => {
  document
    .getElementById("msg-send-btn")
    .addEventListener("click", async () => {
      let content = document.getElementById("msg-input").value;
      let ret = await call_message(content);
      console.log(ret);
    });
})();
